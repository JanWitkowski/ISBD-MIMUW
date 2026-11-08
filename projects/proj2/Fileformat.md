# Format

## Structure
PAX layout
+ File
    + File Magic  4B 55 50 41
    + Row Group 0 
        + COLUMN 0
            + Page 0 metadata | page data
            + Page 1 metadata | page data
            + Page 2 metadata | page data
            + ...
        + COLUMN 1
            + Page 0 metadata | page data
            + Page 1 metadata | page data
            + Page 2 metadata | page data
            + ...
        + ...
    + Row Group 1
        + COLUMN 0
            + Page 0 metadata | page data
            + Page 1 metadata | page data
            + Page 2 metadata | page data
            + ...
        + COLUMN 1
            + Page 0 metadata | page data
            + Page 1 metadata | page data
            + Page 2 metadata | page data
            + ...
        + ...
    + ...
    + File Footer
    + Footer Length uint64
    + EOF

## Metadata

### File Footer
- Schema {
    COLUMN_COUNT uint64 not infinte but quite damn close
    COLUMN_COUNT * {
        NAME_LENGTH uint32
        NAME byte[NAME_LENGTH]
        Type byte
    }
}
- NUM_ROWS u64
- RG_COUNT u64
- RG_META RGCOUNT * {
    FIRST_ROW   uint64
    ROW_COUNT   uint64

    COLUMN_CHUNKS_Meta   COLUMN_COUNT * {
        FILE_OFFSET u64
        FULL_SIZE   u64
        ENCODED_SIZE    u64
        PAGE_COUNT  u64
        PAGE_INDEX  PAGE_COUNT * {
            FILE_OFFSET u64
            PAGE_SIZE   u32
            FIRST_ROW   u16
            LAST_ROW    u16
        }
    }
}

### Type
enum class DataType : uint8_t {
    INT64   = 1,
    BOOLEAN = 2,
    VARCHAR = 3
};

### Page Header


## Big VARCHAR handling
across multiple pages 

## Sizes
RG_TARGET_SIZE = 16 777 216  16 MiB (parquet has 128)

RG_MAX_ROWS = 32 768

for estimation common row 3 bool 4 int 3 varchar(10) ~ 343 Bytes per row

TARGET_PAGE_SIZE = 65 536 - LZ4 sweet spot

~256 pages per row group 

big VARCHAR handling many pages with metadata
[fragment_length] [has_more: 1 bit] [fragment_data]




Batch - bounded column data ready for processing 
Batch_SIZE=1024 - number of rows in a batch(all with unlimited columns)


## Column datatypes compression
- signed int_64 (delta-encoding->ZigZag->VLE)
- boolean (1 bit)
- VARCHAR unlimited length (LZ4)

