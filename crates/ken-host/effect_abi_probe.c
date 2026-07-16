#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

struct SliceV1 { const uint8_t *data; size_t len; };
struct CapabilityTokenV1 { uint32_t slot; uint32_t generation; };
struct NativeInvocationV1 { const void *process_input; void *host_context; uint64_t capability; };
struct HostInitResultV1 { void *context; uint64_t capability; uint64_t plan_hash; };
struct KenBorrowedValue { uint64_t kind; uint64_t tag; const void *data; size_t len; };
struct ConsoleWriteRequestV1 { uint64_t stream; struct SliceV1 bytes; };
struct ConsoleReadRequestV1 { uint64_t stream; uint64_t limit; };
struct ConsoleStreamRequestV1 { uint64_t stream; };
struct UnitRequestV1 { uint8_t reserved; };
struct FsReadFileRequestV1 { uint64_t capability; struct SliceV1 path; };
struct FsWriteFileRequestV1 {
    uint64_t capability;
    struct SliceV1 path;
    uint64_t create_policy;
    struct SliceV1 bytes;
};
struct FsAppendFileRequestV1 { uint64_t capability; struct SliceV1 path; struct SliceV1 bytes; };
struct FsPathRequestV1 { uint64_t capability; struct SliceV1 path; };
struct FsRecursivePathRequestV1 { uint64_t capability; uint64_t recursive; struct SliceV1 path; };
struct FsRenameRequestV1 { uint64_t capability; struct SliceV1 source; struct SliceV1 destination; };
struct HostReplyV1 { uint64_t tag; uint64_t detail; struct SliceV1 bytes; };

#define FACT_SIZE(T) printf("SIZE_%s=%zu\n", #T, sizeof(struct T))
#define FACT_ALIGN(T) printf("ALIGN_%s=%zu\n", #T, _Alignof(struct T))
#define FACT_OFFSET(T, F) printf("OFFSET_%s_%s=%zu\n", #T, #F, offsetof(struct T, F))

int main(void) {
    FACT_SIZE(SliceV1); FACT_ALIGN(SliceV1); FACT_OFFSET(SliceV1, data); FACT_OFFSET(SliceV1, len);
    FACT_SIZE(CapabilityTokenV1); FACT_ALIGN(CapabilityTokenV1); FACT_OFFSET(CapabilityTokenV1, slot); FACT_OFFSET(CapabilityTokenV1, generation);
    FACT_SIZE(NativeInvocationV1); FACT_ALIGN(NativeInvocationV1); FACT_OFFSET(NativeInvocationV1, process_input); FACT_OFFSET(NativeInvocationV1, host_context); FACT_OFFSET(NativeInvocationV1, capability);
    FACT_SIZE(HostInitResultV1); FACT_ALIGN(HostInitResultV1); FACT_OFFSET(HostInitResultV1, context); FACT_OFFSET(HostInitResultV1, capability); FACT_OFFSET(HostInitResultV1, plan_hash);
    FACT_SIZE(KenBorrowedValue); FACT_ALIGN(KenBorrowedValue); FACT_OFFSET(KenBorrowedValue, kind); FACT_OFFSET(KenBorrowedValue, tag); FACT_OFFSET(KenBorrowedValue, data); FACT_OFFSET(KenBorrowedValue, len);
    FACT_SIZE(ConsoleWriteRequestV1); FACT_ALIGN(ConsoleWriteRequestV1); FACT_OFFSET(ConsoleWriteRequestV1, stream); FACT_OFFSET(ConsoleWriteRequestV1, bytes);
    FACT_SIZE(ConsoleReadRequestV1); FACT_ALIGN(ConsoleReadRequestV1); FACT_OFFSET(ConsoleReadRequestV1, stream); FACT_OFFSET(ConsoleReadRequestV1, limit);
    FACT_SIZE(ConsoleStreamRequestV1); FACT_ALIGN(ConsoleStreamRequestV1); FACT_OFFSET(ConsoleStreamRequestV1, stream);
    FACT_SIZE(UnitRequestV1); FACT_ALIGN(UnitRequestV1); FACT_OFFSET(UnitRequestV1, reserved);
    FACT_SIZE(FsReadFileRequestV1); FACT_ALIGN(FsReadFileRequestV1); FACT_OFFSET(FsReadFileRequestV1, capability); FACT_OFFSET(FsReadFileRequestV1, path);
    FACT_SIZE(FsWriteFileRequestV1); FACT_ALIGN(FsWriteFileRequestV1); FACT_OFFSET(FsWriteFileRequestV1, capability); FACT_OFFSET(FsWriteFileRequestV1, path); FACT_OFFSET(FsWriteFileRequestV1, create_policy); FACT_OFFSET(FsWriteFileRequestV1, bytes);
    FACT_SIZE(FsAppendFileRequestV1); FACT_ALIGN(FsAppendFileRequestV1); FACT_OFFSET(FsAppendFileRequestV1, capability); FACT_OFFSET(FsAppendFileRequestV1, path); FACT_OFFSET(FsAppendFileRequestV1, bytes);
    FACT_SIZE(FsPathRequestV1); FACT_ALIGN(FsPathRequestV1); FACT_OFFSET(FsPathRequestV1, capability); FACT_OFFSET(FsPathRequestV1, path);
    FACT_SIZE(FsRecursivePathRequestV1); FACT_ALIGN(FsRecursivePathRequestV1); FACT_OFFSET(FsRecursivePathRequestV1, capability); FACT_OFFSET(FsRecursivePathRequestV1, recursive); FACT_OFFSET(FsRecursivePathRequestV1, path);
    FACT_SIZE(FsRenameRequestV1); FACT_ALIGN(FsRenameRequestV1); FACT_OFFSET(FsRenameRequestV1, capability); FACT_OFFSET(FsRenameRequestV1, source); FACT_OFFSET(FsRenameRequestV1, destination);
    FACT_SIZE(HostReplyV1); FACT_ALIGN(HostReplyV1); FACT_OFFSET(HostReplyV1, tag); FACT_OFFSET(HostReplyV1, detail); FACT_OFFSET(HostReplyV1, bytes);
    return 0;
}
