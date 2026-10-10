#include "BinaryReader.h"
#include <bit>
#include <cstring>
#include <istream>
#include <stdexcept>

using namespace WallpaperEngine::Data::Utils;

BinaryReader::BinaryReader(ReadStreamSharedPtr file) : m_input(std::move(file)) {
    if (!m_input) throw std::invalid_argument("Missing binary stream");
}
uint32_t BinaryReader::nextUInt32() const {
    unsigned char bytes[4];
    next(reinterpret_cast<char*>(bytes), sizeof(bytes));
    return uint32_t(bytes[0]) | (uint32_t(bytes[1]) << 8)
        | (uint32_t(bytes[2]) << 16) | (uint32_t(bytes[3]) << 24);
}
int BinaryReader::nextInt() const { return std::bit_cast<int32_t>(nextUInt32()); }
float BinaryReader::nextFloat() const { return std::bit_cast<float>(nextUInt32()); }
std::string BinaryReader::nextNullTerminatedString() const {
    std::string result;
    for (;;) {
        const char byte = next();
        if (!byte) return result;
        if (result.size() >= 1024 * 1024) throw std::runtime_error("Binary string exceeds budget");
        result += byte;
    }
}
std::string BinaryReader::nextSizedString() const {
    const uint32_t length = nextUInt32();
    if (length > 1024 * 1024) throw std::runtime_error("Binary string exceeds budget");
    std::string result(length, '\0');
    next(result.data(), length);
    return result;
}
void BinaryReader::next(char* out, size_t size) const {
    if (size > 256 * 1024 * 1024) throw std::runtime_error("Binary read exceeds budget");
    m_input->read(out, static_cast<std::streamsize>(size));
    if (m_input->gcount() != static_cast<std::streamsize>(size))
        throw std::runtime_error("Truncated binary resource");
}
char BinaryReader::next() const { char byte; next(&byte, 1); return byte; }
std::istream& BinaryReader::base() const { return *m_input; }
