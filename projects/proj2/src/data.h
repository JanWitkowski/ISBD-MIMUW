#pragma once 

#include <cstdint>
#include <vector>
#include <string>
#include <variant>

enum class ColumnType : uint8_t {
    INT = 0,
    BOOLEAN = 1,
    VARCHAR = 2
};

struct ColumnSchema {
    ColumnType type;
    std::string name;
};


struct Schema {
    std::vector<ColumnSchema> columns;
};

using Column = std::variant<
    std::vector<int64_t>,
    std::vector<uint8_t>,
    std::vector<std::string>
>;

struct Batch {
    uint32_t row_count;
    std::vector<Column> columns;
};


