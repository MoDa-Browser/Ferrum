#pragma once

#include <string>
#include <vector>
#include <memory>

namespace moda::render {

struct Rect {
    float x;
    float y;
    float width;
    float height;

    Rect(float x, float y, float width, float height)
        : x(x), y(y), width(width), height(height) {}
};

struct BoxModel {
    float margin_top;
    float margin_right;
    float margin_bottom;
    float margin_left;
    float padding_top;
    float padding_right;
    float padding_bottom;
    float padding_left;
    float border_top;
    float border_right;
    float border_bottom;
    float border_left;

    BoxModel()
        : margin_top(0), margin_right(0), margin_bottom(0), margin_left(0)
        , padding_top(0), padding_right(0), padding_bottom(0), padding_left(0)
        , border_top(0), border_right(0), border_bottom(0), border_left(0) {}
};

class LayoutEngine {
public:
    LayoutEngine();
    ~LayoutEngine();

    void calculate_layout(const std::string& html);
    Rect get_element_bounds(const std::string& element_id) const;
    BoxModel get_element_box_model(const std::string& element_id) const;

private:
    class Impl;
    std::unique_ptr<Impl> impl_;
};

} // namespace moda::render
