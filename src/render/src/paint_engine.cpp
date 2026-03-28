#include "paint_engine.hpp"

namespace moda::render {

class PaintEngine::Impl {
public:
    Impl() = default;
    ~Impl() = default;
};

PaintEngine::PaintEngine()
    : impl_(std::make_unique<Impl>()) {}

PaintEngine::~PaintEngine() = default;

void PaintEngine::begin_frame() {
}

void PaintEngine::end_frame() {
}

void PaintEngine::draw_rect(const Rect& rect, const Color& color) {
}

void PaintEngine::draw_text(const std::string& text, float x, float y, const Color& color) {
}

void PaintEngine::draw_image(const std::string& image_path, const Rect& rect) {
}

void PaintEngine::set_clip_rect(const Rect& rect) {
}

void PaintEngine::clear_clip_rect() {
}

} // namespace moda::render
