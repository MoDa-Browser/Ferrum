#pragma once

#include <string>
#include <vector>
#include <memory>

namespace moda::render {

enum class NodeType {
    Element,
    Text,
    Comment,
};

struct DOMNode {
    NodeType type;
    std::string tag_name;
    std::string text_content;
    std::vector<std::shared_ptr<DOMNode>> children;
    std::shared_ptr<DOMNode> parent;

    DOMNode(NodeType type) : type(type) {}
};

class DOMManager {
public:
    DOMManager();
    ~DOMManager();

    std::shared_ptr<DOMNode> parse_html(const std::string& html);
    std::shared_ptr<DOMNode> get_element_by_id(const std::string& id) const;
    std::vector<std::shared_ptr<DOMNode>> get_elements_by_tag_name(
        const std::string& tag_name) const;

    void update_dom(const std::string& html);
    void clear();

private:
    class Impl;
    std::unique_ptr<Impl> impl_;
};

} // namespace moda::render
