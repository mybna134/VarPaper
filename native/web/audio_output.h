// Per-browser bounded PCM output for generated HTML and Web Audio sound.
#pragma once
#include <SDL.h>
#include <algorithm>
#include <atomic>
#include <cmath>
#include <deque>
#include <mutex>
#include <stdexcept>
#include <string>

class AudioOutput {
public:
  ~AudioOutput() { stop(); }
  void start() {
    std::lock_guard lifecycle(lifecycle_);
    close_device();
    {
      std::lock_guard lock(initialization());
      if (SDL_InitSubSystem(SDL_INIT_AUDIO) != 0)
        throw std::runtime_error(std::string("Audio output unavailable: ") +
                                 SDL_GetError());
    }
    SDL_AudioSpec wanted{};
    wanted.freq = 48000;
    wanted.format = AUDIO_F32SYS;
    wanted.channels = 2;
    wanted.samples = 512;
    wanted.callback = callback;
    wanted.userdata = this;
    device_ = SDL_OpenAudioDevice(nullptr, 0, &wanted, nullptr, 0);
    if (!device_) {
      {
        std::lock_guard lock(initialization());
        SDL_QuitSubSystem(SDL_INIT_AUDIO);
      }
      throw std::runtime_error(std::string("Cannot open audio output: ") +
                               SDL_GetError());
    }
    active_.store(true);
    SDL_PauseAudioDevice(device_, 0);
  }
  void stop() {
    std::lock_guard lifecycle(lifecycle_);
    close_device();
  }
  void configure(bool muted, float volume) {
    muted_.store(muted);
    volume_.store(std::clamp(volume, 0.0f, 1.0f));
  }
  void packet(const float **planar, int frames) {
    if (!planar || !planar[0] || !planar[1] || frames <= 0 || frames > 4800)
      throw std::runtime_error("Invalid browser audio packet");
    std::lock_guard lock(queue_mutex_);
    if (!active_.load())
      return;
    // At most 250 ms; discard oldest complete stereo frames on overload.
    while (queue_.size() + size_t(frames) * 2 > 24000) {
      queue_.pop_front();
      queue_.pop_front();
    }
    for (int i = 0; i < frames; ++i)
      for (int channel = 0; channel < 2; ++channel) {
        float sample = planar[channel][i];
        queue_.push_back(std::isfinite(sample) ? std::clamp(sample, -1.0f, 1.0f)
                                               : 0.0f);
      }
    packets_.fetch_add(1);
  }
  void mix(float *output, size_t samples) {
    std::fill(output, output + samples, 0.0f);
    std::lock_guard lock(queue_mutex_);
    float gain = muted_.load() ? 0.0f : volume_.load();
    for (size_t i = 0; i < samples && !queue_.empty(); ++i) {
      output[i] = queue_.front() * gain;
      queue_.pop_front();
      peak_.store(std::max(peak_.load(), std::abs(output[i])));
    }
  }
  uint64_t packets() const { return packets_.load(); }
  float peak() { return peak_.exchange(0); }

private:
  static std::mutex &initialization() {
    static std::mutex mutex;
    return mutex;
  }
  static void callback(void *self, Uint8 *output, int bytes) {
    static_cast<AudioOutput *>(self)->mix(reinterpret_cast<float *>(output),
                                          size_t(bytes) / sizeof(float));
  }
  void close_device() {
    active_.store(false);
    if (device_) {
      SDL_CloseAudioDevice(device_);
      device_ = 0;
      std::lock_guard lock(initialization());
      SDL_QuitSubSystem(SDL_INIT_AUDIO);
    }
    std::lock_guard lock(queue_mutex_);
    queue_.clear();
  }
  std::mutex lifecycle_, queue_mutex_;
  SDL_AudioDeviceID device_ = 0;
  std::deque<float> queue_;
  std::atomic<bool> active_{false}, muted_{true};
  std::atomic<float> volume_{1.0f}, peak_{0.0f};
  std::atomic<uint64_t> packets_{0};
};
