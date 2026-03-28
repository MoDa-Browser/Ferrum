#pragma once

#include "layout_engine.hpp"
#include <string>
#include <vector>

namespace moda::render {

struct Color {
    float r;
    float g;
    float b;
    float a;

    Color(float r, float g, float b, float a = 1.0f)
        : r(r), g(g), b(b), a(a) {}
};

class PaintEngine {
public:
    PaintEngine();
    ~PaintEngine();

    void begin_frame();
    void end_frame();

    void draw_rect(const Rect& rect, const Color& color);
    void draw_text(const std::string& text, float x, float y, const Color& color);
    void draw_image(const std::string& image_path, const Rect& rect);

    void set_clip_rect(const Rect& rect);
    void clear_clip_rect();

private:
    class Impl;
    std::unique_ptr<Impl> impl_;
};

} // namespace moda::render
