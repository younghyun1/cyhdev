# Media persistence

Image endpoints stage multipart file fields to automatically removed temporary files through a 64 KiB userspace buffer. The staged encoded input is never accumulated into a request-sized `Vec<u8>`. EXIF reads from the staged path, processed AVIF output is written to temporary files through another 64 KiB buffer, and S3 reads those files as path-backed byte streams.

Image decoding is globally limited to two concurrent jobs. Each source is rejected above 16,384 pixels on either axis or 64 Mi pixels in total, and the decoder receives a 256 MiB allocation limit. Photograph variants are produced largest-to-smallest from one decoded image, so thumbnail creation does not decode or clone the original a second time. Batch processing additionally limits end-to-end item concurrency to four, bounding simultaneous object-store and database work.

Object-store and database ordering is centralized in `util::media::persistence`. New objects upload first. If a later upload or the database commit fails, every new object uploaded by that attempt is deleted in reverse order. Compensation failures are returned with the original failure and retain an explicit retry-safe classification.

Profile-picture replacement locks the account row, inserts the new metadata, and retires prior active links in one database transaction. Only after that transaction commits does the persistence workflow delete superseded objects. A failed superseded-object deletion does not invalidate the committed replacement or produce a misleading client retry; it is returned as explicit cleanup work and logged with its object key and retry classification.

`tests/media_cleanup_compensation.rs` uses the object-store interface with a recording fake. It verifies database-failure cleanup, partial-upload cleanup, and the rule that superseded objects are not deleted until after the metadata commit.
