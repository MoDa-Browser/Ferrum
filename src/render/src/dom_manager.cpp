#include "dom_manager.hpp"

namespace moda::render {

class DOMManager::Impl {
public:
    Impl() = default;
    ~Impl() = default;
};

DOMManager::DOMManager()
    : impl_(std::make_unique<Impl>()) {}

DOMManager::~DOMManager() = default;

std::shared_ptr<DOMNode> DOMManager::parse_html(const std::string& html) {
    return std::make_shared<DOMNode>(NodeType::Element);
}

std::shared_ptr<DOMNode> DOMManager::get_element_by_id(const std::string& id) const {
    return nullptr;
}

std::vector<std::shared_ptr<DOMNode>> DOMManager::get_elements_by_tag_name(
    const std::string& tag_name) const {
    return {};
}

void DOMManager::update_dom(const std::string& html) {
}

void DOMManager::clear() {
}

} // namespace moda::render
