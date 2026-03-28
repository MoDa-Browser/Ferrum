#include "layout_engine.hpp"
#include "paint_engine.hpp"
#include "dom_manager.hpp"
#include <iostream>

using namespace moda::render;

int main() {
    std::cout << "MoDa Browser Core - Minimal Browser Example" << std::endl;
    std::cout << "============================================" << std::endl;

    DOMManager dom_manager;
    LayoutEngine layout_engine;
    PaintEngine paint_engine;

    std::string html = "<html><body><h1>Hello, MoDa!</h1></body></html>";

    auto dom = dom_manager.parse_html(html);
    if (dom) {
        std::cout << "DOM parsed successfully" << std::endl;
    }

    layout_engine.calculate_layout(html);
    std::cout << "Layout calculated" << std::endl;

    paint_engine.begin_frame();
    paint_engine.draw_rect(Rect(10, 10, 100, 100), Color(1.0, 0.0, 0.0));
    paint_engine.draw_text("Hello, MoDa!", 10, 10, Color(1.0, 1.0, 1.0));
    paint_engine.end_frame();

    std::cout << "Rendering complete" << std::endl;

    return 0;
}
