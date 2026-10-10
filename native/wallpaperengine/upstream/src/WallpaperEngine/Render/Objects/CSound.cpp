#include <SDL.h>
#include <memory>

#include "CSound.h"
#include "WallpaperEngine/Render/Wallpapers/CScene.h"

#include "WallpaperEngine/FileSystem/Container.h"

using namespace WallpaperEngine::Render::Objects;

CSound::CSound (Wallpapers::CScene& scene, const Sound& sound) : CObject (scene, sound), m_sound (sound) {
    if (this->getContext ().getApp ().getContext ().settings.audio.enabled) {
        try { this->load (); }
        catch (...) {
            for (const auto& [id, stream] : m_audioStreams) {
                getScene().getAudioContext().removeStream(id);
                delete stream;
            }
            throw;
        }
    }
}

CSound::~CSound () {
    // free all the sound buffers and streams
    for (const auto& stream : this->m_audioStreams) {
	this->getScene ().getAudioContext ().removeStream (stream.first);
	delete stream.second;
    }

    this->m_audioStreams.clear ();
}

void CSound::load () {
    for (const auto& cur : this->m_sound.sounds) {
	auto stream = std::make_unique<Audio::AudioStream>(this->getScene().getAudioContext(), this->getAssetLocator().read(cur),
            m_sound.playbackmode.has_value() && m_sound.playbackmode == "loop");


	// add the stream to the context so it can be played
	const int id = getScene().getAudioContext().addStream(stream.get());
        try { m_audioStreams.emplace(id, stream.get()); }
        catch (...) { getScene().getAudioContext().removeStream(id); throw; }
        stream.release();
    }
}

void CSound::render () { }
