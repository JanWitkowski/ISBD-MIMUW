#pragma once 

#include "data.h"
#include <array>

namespace Parkiet {

    inline constexpr std::array<char, 4> MAGIC{
        '\x4B', '\x55', '\x50', '\x41'
    };
    
    inline constexpr uint64_t RG_TARGET_SIZE = 16777216;
    inline constexpr uint32_t RG_MAX_ROWS = 32768;
    inline constexpr uint32_t PAGE_TARGET_SIZE = 65536;
    inline constexpr uint32_t BATCH_SIZE = 1024;

    struct PageIndexEntry {
        uint64_t file_offset;
        uint32_t page_size;
        uint16_t first_row;
        uint16_t last_row;
    };

    struct ColumnChunkMeta {
        uint64_t file_offset;
        uint64_t full_size;
        uint64_t encoded_size;
        std::vector<PageIndexEntry> page_index;
    };

    struct RowGroupMeta {
        uint64_t first_row;
        uint64_t row_count;
        std::vector<ColumnChunkMeta> column_chunks;
    };

    struct FileMeta {
        Schema schema;
        uint64_t num_rows;
        std::vector<RowGroupMeta> rg_meta;
    };

}