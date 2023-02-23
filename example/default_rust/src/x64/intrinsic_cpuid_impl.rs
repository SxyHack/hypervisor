extern "C" {
    /// <!-- description -->
    ///   @brief Executes the CPUID instruction given the provided EAX and ECX
    ///     and returns the results
    ///
    /// <!-- inputs/outputs -->
    ///   @param gs ignored
    ///   @param rax the index used by CPUID, returns resulting rax
    ///   @param rbx returns resulting rbx
    ///   @param rcx the subindex used by CPUID, returns the resulting rcx
    ///   @param rdx returns resulting rdx
    ///
    pub fn intrinsic_cpuid_impl(rax: *mut u64, rbx: *mut u64, rcx: *mut u64, rdx: *mut u64);
}
