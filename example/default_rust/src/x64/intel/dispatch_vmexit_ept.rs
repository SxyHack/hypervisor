/// handle EPT_VIOLATION
pub fn dispatch_vmexit_ept_violation(
    gs: &crate::GsT,
    tls: &crate::TlsT,
    sys: &mut syscall::BfSyscallT,
    intrinsic: &crate::IntrinsicT,
    vsid: bsl::SafeU16,
) -> bsl::ErrcType {
    bsl::discard(gs);
    bsl::discard(tls);
    bsl::discard(sys);
    bsl::discard(intrinsic);
    bsl::discard(vsid);

    bsl::errc_success
}


/// handle EPT_MISCONFIG
pub fn dispatch_vmexit_ept_misconfig(
    gs: &crate::GsT,
    tls: &crate::TlsT,
    sys: &mut syscall::BfSyscallT,
    intrinsic: &crate::IntrinsicT,
    vsid: bsl::SafeU16,
) -> bsl::ErrcType {
    bsl::discard(gs);
    bsl::discard(tls);
    bsl::discard(sys);
    bsl::discard(intrinsic);
    bsl::discard(vsid);

    bsl::errc_success
}
