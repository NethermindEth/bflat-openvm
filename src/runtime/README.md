# OpenVM runtime

[![Nethermind.OpenVm.Runtime](https://img.shields.io/nuget/v/Nethermind.OpenVm.Runtime)](https://www.nuget.org/packages/Nethermind.OpenVm.Runtime)

OpenVM zkVM accelerators for the Nethermind guest for [OpenVM](https://github.com/openvm-org/openvm),
consumed by [bflat-riscv64](https://github.com/NethermindEth/bflat-riscv64)
guests through `--extlib` and called from managed code through
[Nethermind.Zkvm.Abstractions](https://www.nuget.org/packages/Nethermind.Zkvm.Abstractions).

The package ships `libopenvm.a` together with the `*.bflat.manifest` that tells
bflat which target triple it belongs to.

## License

This package contains only code written for this repository, licensed
under the [MIT](https://github.com/NethermindEth/bflat-openvm/blob/main/LICENSE)
license. The OpenVM instruction encodings it targets come from
[OpenVM](https://github.com/openvm-org/openvm), which is Apache-2.0/MIT
dual-licensed.
