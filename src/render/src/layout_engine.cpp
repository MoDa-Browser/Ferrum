#include "layout_engine.hpp"

namespace moda::render {

class LayoutEngine::Impl {
public:
    Impl() = default;
    ~Impl() = default;
};

LayoutEngine::LayoutEngine()
    : impl_(std::make_unique<Impl>()) {}

LayoutEngine::~LayoutEngine() = default;

void LayoutEngine::calculate_layout(const std::string& html) {
}

Rect LayoutEngine::get_element_bounds(const std::string& element_id) const {
    return Rect(0, 0, 0, 0);
}

BoxModel LayoutEngine::get_element_box_model(const std::string& element_id) const {
    return BoxModel();
}

} // namespace moda::render
