#include <archive.h>
#include <archive_entry.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    struct archive* reader = archive_read_new();
    archive_read_support_filter_all(reader);
    archive_read_support_format_raw(reader);
    archive_read_support_format_all(reader);
    archive_read_open_filename(reader, "/tmp/test.zip", 1);

    struct archive_entry* entry = NULL;
    while (1) {
        if (archive_read_next_header(reader, &entry) == ARCHIVE_EOF) {
            break;
        }
        const char* pathname = archive_entry_pathname(entry);
        if (strcmp(pathname, "content/nested/second") == 0) {
            break;
        }
    }

    const void* buffer = NULL;
    size_t size = 0;
    off_t offset = 0;
    archive_read_data_block(reader, &buffer, &size, &offset);
    puts((char *)buffer);

    archive_read_close(reader);
    archive_read_free(reader);
    return 0;
}
