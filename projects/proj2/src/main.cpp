


// serialize
int main() {
    Schema schema = source.read_schema();

    Serializer serializer("output.parkiet", schema);

    while (source.has_next()) {
        Batch batch = source.next_batch();
        serializer.append(batch);
    }

    serializer.finish();

    return 0;
}

