/**
 * @copyright
 * Copyright (C) 2020 Assured Information Security, Inc.
 *
 * @copyright
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * @copyright
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * @copyright
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#include <alloc_and_copy_ext_elf_files_from_user.h>
#include <alloc_and_copy_mk_elf_file_from_user.h>
#include <alloc_and_copy_mk_elf_segments.h>
#include <alloc_mk_huge_pool.h>
#include <alloc_mk_page_pool.h>
#include <alloc_mk_root_page_table.h>
#include <debug.h>
#include <debug_ring_t.h>
#include <dump_ext_elf_files.h>
#include <dump_mk_elf_file.h>
#include <dump_mk_elf_segments.h>
#include <dump_mk_huge_pool.h>
#include <dump_mk_page_pool.h>
#include <dump_mk_root_page_table.h>
#include <g_mut_ext_elf_files.h>
#include <g_mut_mk_code_aliases.h>
#include <g_mut_mk_elf_file.h>
#include <g_mut_mk_elf_segments.h>
#include <g_mut_mk_huge_pool.h>
#include <g_mut_mk_page_pool.h>
#include <g_mut_vmm_status.h>
#include <g_pmut_mut_mk_debug_ring.h>
#include <g_pmut_mut_mk_root_page_table.h>
#include <map_ext_elf_files.h>
#include <map_mk_code_aliases.h>
#include <map_mk_debug_ring.h>
#include <map_mk_elf_file.h>
#include <map_mk_elf_segments.h>
#include <map_mk_huge_pool.h>
#include <map_mk_page_pool.h>
#include <platform.h>
#include <span_t.h>
#include <start_vmm_args_t.h>
#include <start_vmm_per_cpu.h>
#include <stop_and_free_the_vmm.h>
#include <types.h>

/**
 * <!-- description -->
 *   @brief Allocates and starts the VMM
 *
 * <!-- inputs/outputs -->
 *   @param args the arguments to verify
 *   @return LOADER_SUCCESS on success, LOADER_FAILURE on failure.
 */
NODISCARD static int64_t
alloc_and_start_the_vmm(struct start_vmm_args_t const *const args) NOEXCEPT
{
    if (VMM_STATUS_CORRUPT == g_mut_vmm_status) {
        bferror("Unable to start, previous VMM failed to properly stop");
        return LOADER_FAILURE;
    }

    if (VMM_STATUS_RUNNING == g_mut_vmm_status) {
        stop_and_free_the_vmm();
    }
    else {
        bf_touch();
    }

    g_pmut_mut_mk_debug_ring->epos = ((uint64_t)0);
    g_pmut_mut_mk_debug_ring->spos = ((uint64_t)0);

    if (alloc_mk_root_page_table(&g_pmut_mut_mk_root_page_table)) {
        bferror("alloc_mk_root_page_table failed");
        goto alloc_mk_root_page_table_failed;
    }

    if (alloc_and_copy_mk_elf_file_from_user(&args->mk_elf_file, &g_mut_mk_elf_file)) {
        bferror("alloc_and_copy_mk_elf_file_from_user failed");
        goto alloc_and_copy_mk_elf_file_from_user_failed;
    }

    if (alloc_and_copy_ext_elf_files_from_user(args->ext_elf_files, g_mut_ext_elf_files)) {
        bferror("alloc_and_copy_ext_elf_files_from_user failed");
        goto alloc_and_copy_ext_elf_files_from_user_failed;
    }

    if (alloc_and_copy_mk_elf_segments(&g_mut_mk_elf_file, g_mut_mk_elf_segments)) {
        bferror("alloc_and_copy_mk_elf_segments failed");
        goto alloc_and_copy_mk_elf_segments_failed;
    }

    if (alloc_mk_page_pool(args->num_pages_in_page_pool, &g_mut_mk_page_pool)) {
        bferror("alloc_mk_page_pool failed");
        goto alloc_mk_page_pool_failed;
    }

    if (alloc_mk_huge_pool(0U, &g_mut_mk_huge_pool)) {
        bferror("alloc_mk_huge_pool failed");
        goto alloc_mk_huge_pool_failed;
    }

    if (map_mk_debug_ring(g_pmut_mut_mk_debug_ring, g_pmut_mut_mk_root_page_table)) {
        bferror("map_mk_debug_ring failed");
        goto map_mk_debug_ring_failed;
    }

    if (map_mk_code_aliases(&g_mut_mk_code_aliases, g_pmut_mut_mk_root_page_table)) {
        bferror("map_mk_code_aliases failed");
        goto map_mk_code_aliases_failed;
    }

    if (map_mk_elf_file(&g_mut_mk_elf_file, g_pmut_mut_mk_root_page_table)) {
        bferror("map_mk_elf_file failed");
        goto map_mk_elf_file_failed;
    }

    if (map_ext_elf_files(g_mut_ext_elf_files, g_pmut_mut_mk_root_page_table)) {
        bferror("map_ext_elf_files failed");
        goto map_ext_elf_files_failed;
    }

    if (map_mk_elf_segments(g_mut_mk_elf_segments, g_pmut_mut_mk_root_page_table)) {
        bferror("map_mk_elf_segments failed");
        goto map_mk_elf_segments_failed;
    }

    if (map_mk_page_pool(&g_mut_mk_page_pool, g_pmut_mut_mk_root_page_table)) {
        bferror("map_mk_page_pool failed");
        goto map_mk_page_pool_failed;
    }

    if (map_mk_huge_pool(&g_mut_mk_huge_pool, g_pmut_mut_mk_root_page_table)) {
        bferror("map_mk_huge_pool failed");
        goto map_mk_huge_pool_failed;
    }

#ifdef DEBUG_LOADER
    dump_mk_root_page_table(g_pmut_mut_mk_root_page_table);
    dump_mk_elf_file(&g_mut_mk_elf_file);
    dump_ext_elf_files(g_mut_ext_elf_files);
    dump_mk_elf_segments(g_mut_mk_elf_segments);
    dump_mk_page_pool(&g_mut_mk_page_pool);
    dump_mk_huge_pool(&g_mut_mk_huge_pool);
#endif

    if (platform_on_each_cpu(start_vmm_per_cpu, PLATFORM_PARALLEL)) {
        bferror("start_vmm_per_cpu failed");
        goto start_vmm_per_cpu_failed;
    }

    g_mut_vmm_status = VMM_STATUS_RUNNING;
    return LOADER_SUCCESS;

start_vmm_per_cpu_failed:
map_mk_huge_pool_failed:
map_mk_page_pool_failed:
map_mk_elf_segments_failed:
map_ext_elf_files_failed:
map_mk_elf_file_failed:
map_mk_code_aliases_failed:
map_mk_debug_ring_failed:
alloc_mk_huge_pool_failed:
alloc_mk_page_pool_failed:
alloc_and_copy_mk_elf_segments_failed:
alloc_and_copy_ext_elf_files_from_user_failed:
alloc_and_copy_mk_elf_file_from_user_failed:
alloc_mk_root_page_table_failed:

    stop_and_free_the_vmm();
    return LOADER_FAILURE;
}

/**
 * <!-- description -->
 *   @brief Verifies that the arguments from the IOCTL are valid.
 *
 * <!-- inputs/outputs -->
 *   @param args the arguments to verify
 *   @return LOADER_SUCCESS on success, LOADER_FAILURE on failure.
 */
NODISCARD static int64_t
verify_start_vmm_args(struct start_vmm_args_t const *const args) NOEXCEPT
{
    uint64_t mut_i;

    if (((uint64_t)1) != args->ver) {
        bferror("IOCTL ABI version not supported");
        return LOADER_FAILURE;
    }

    if (NULLPTR == args->mk_elf_file.addr) {
        bferror("the microkernel is required");
        return LOADER_FAILURE;
    }

    if (((uint64_t)0) == args->mk_elf_file.size) {
        bferror("mk_elf_file.size is invalid");
        return LOADER_FAILURE;
    }

    if (HYPERVISOR_MAX_ELF_FILE_SIZE <= args->mk_elf_file.size) {
        bferror("mk_elf_file.size is invalid");
        return LOADER_FAILURE;
    }

    if (NULLPTR == args->ext_elf_files[((uint64_t)0)].addr) {
        bferror("at least one extension is required");
        return LOADER_FAILURE;
    }

    for (mut_i = ((uint64_t)0); mut_i < HYPERVISOR_MAX_EXTENSIONS; ++mut_i) {
        if (NULLPTR == args->ext_elf_files[mut_i].addr) {
            if (((uint64_t)0) != args->ext_elf_files[mut_i].size) {
                bferror("invalid extension address/size combination");
                return LOADER_FAILURE;
            }

            bf_touch();
        }
        else {
            bf_touch();
        }

        if (((uint64_t)0) == args->ext_elf_files[mut_i].size) {
            if (NULLPTR != args->ext_elf_files[mut_i].addr) {
                bferror("invalid extension address/size combination");
                return LOADER_FAILURE;
            }

            bf_touch();
        }
        else {
            bf_touch();
        }

        if (HYPERVISOR_MAX_ELF_FILE_SIZE <= args->ext_elf_files[mut_i].size) {
            bferror_d32("ext_elf_files.size is invalid", (uint32_t)mut_i);
            return LOADER_FAILURE;
        }

        bf_touch();
    }

    return LOADER_SUCCESS;
}

/**
 * <!-- description -->
 *   @brief This function contains all of the code that is common between
 *     all archiectures and all platforms for starting the VMM. This function
 *     will call platform and architecture specific functions as needed.
 *
 * <!-- inputs/outputs -->
 *   @param args arguments from the ioctl
 *   @return LOADER_SUCCESS on success, LOADER_FAILURE on failure.
 */
NODISCARD int64_t
start_vmm(struct start_vmm_args_t const *const args) NOEXCEPT
{
    if (NULLPTR == args) {
        bferror("args was NULL");
        return LOADER_FAILURE;
    }

    if (verify_start_vmm_args(args)) {
        bferror("verify_start_vmm_args failed");
        return LOADER_FAILURE;
    }

    if (alloc_and_start_the_vmm(args)) {
        bferror("alloc_and_start_the_vmm failed");
        return LOADER_FAILURE;
    }

    return LOADER_SUCCESS;
}
