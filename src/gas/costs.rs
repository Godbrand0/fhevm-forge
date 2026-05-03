// FHE operation cost table.
// Coprocessor gas sourced from Zama fhEVM v0.5 documentation.
// On-chain gas is the EVM precompile call overhead (type-independent).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FheType {
    Bool,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    Uint128,
    Uint256,
}

impl FheType {
    pub fn from_sol(s: &str) -> Option<Self> {
        match s {
            "ebool"    => Some(Self::Bool),
            "euint8"   => Some(Self::Uint8),
            "euint16"  => Some(Self::Uint16),
            "euint32"  => Some(Self::Uint32),
            "euint64"  => Some(Self::Uint64),
            "euint128" => Some(Self::Uint128),
            "euint256" => Some(Self::Uint256),
            _          => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Bool    => "ebool",
            Self::Uint8   => "euint8",
            Self::Uint16  => "euint16",
            Self::Uint32  => "euint32",
            Self::Uint64  => "euint64",
            Self::Uint128 => "euint128",
            Self::Uint256 => "euint256",
        }
    }
}

/// Returns (on_chain_gas, coprocessor_gas) for an FHE operation on a given ciphertext type.
/// `op` may include the "TFHE." or "Gateway." prefix.
pub fn fhe_cost(op: &str, ty: FheType) -> (u64, u64) {
    let bare = op
        .strip_prefix("TFHE.")
        .or_else(|| op.strip_prefix("FHE."))
        .or_else(|| op.strip_prefix("Gateway."))
        .unwrap_or(op);

    let on_chain: u64 = match bare {
        "requestDecryption"               => 25_000,
        "allow" | "allowThis"              => 3_000,
        "makePubliclyDecryptable"          => 25_000,
        "isInitialized"                    => 1_000,
        "div" | "rem"                     => 30_000,
        "mul"                             => 15_000,
        "lt"|"le"|"gt"|"ge"|"eq"|"ne"    => 10_000,
        "select" | "min" | "max"          => 12_000,
        "add" | "sub"                     => 8_000,
        "shl" | "shr"                     => 6_000,
        "neg"                             => 5_000,
        "and"|"or"|"xor"|"not"            => 5_000,
        s if s.starts_with("as")          => 6_000,
        _                                 => 5_000,
    };

    let cop: u64 = if bare.starts_with("as") {
        // Cast cost depends on destination type encoded in the op name.
        match bare {
            "asEuint128" | "asEuint256" => match bare {
                "asEuint128" => 55_000,
                _            => 60_000,
            },
            _ => 50_000,
        }
    } else {
        match (bare, ty) {
            // add / sub
            ("add"|"sub", FheType::Uint8|FheType::Uint16|FheType::Uint32|FheType::Uint64) => 65_000,
            ("add"|"sub", FheType::Uint128) => 88_000,
            ("add"|"sub", FheType::Uint256) => 100_000,

            // mul
            ("mul", FheType::Uint8)    => 88_000,
            ("mul", FheType::Uint16)   => 100_000,
            ("mul", FheType::Uint32)   => 120_000,
            ("mul", FheType::Uint64)   => 150_000,
            ("mul", FheType::Uint128)  => 200_000,
            ("mul", FheType::Uint256)  => 500_000,

            // div / rem
            ("div"|"rem", FheType::Uint8)   => 200_000,
            ("div"|"rem", FheType::Uint16)  => 250_000,
            ("div"|"rem", FheType::Uint32)  => 300_000,
            ("div"|"rem", FheType::Uint64)  => 400_000,
            ("div"|"rem", FheType::Uint128) => 600_000,
            ("div"|"rem", FheType::Uint256) => 800_000,

            // bitwise: and / or / xor / not / neg
            ("and"|"or"|"xor"|"not"|"neg", FheType::Bool|FheType::Uint8)  => 26_000,
            ("and"|"or"|"xor"|"not"|"neg", FheType::Uint16) => 27_000,
            ("and"|"or"|"xor"|"not"|"neg", FheType::Uint32) => 28_000,
            ("and"|"or"|"xor"|"not"|"neg", FheType::Uint64) => 30_000,
            ("and"|"or"|"xor"|"not"|"neg", FheType::Uint128)=> 32_000,
            ("and"|"or"|"xor"|"not"|"neg", FheType::Uint256)=> 34_000,

            // shift
            ("shl"|"shr", FheType::Uint8)   => 27_000,
            ("shl"|"shr", FheType::Uint16)  => 28_000,
            ("shl"|"shr", FheType::Uint32)  => 29_000,
            ("shl"|"shr", FheType::Uint64)  => 35_000,
            ("shl"|"shr", FheType::Uint128) => 44_000,
            ("shl"|"shr", FheType::Uint256) => 60_000,

            // min / max
            ("min"|"max", FheType::Uint8)   => 62_000,
            ("min"|"max", FheType::Uint16)  => 64_000,
            ("min"|"max", FheType::Uint32)  => 66_000,
            ("min"|"max", FheType::Uint64)  => 72_000,
            ("min"|"max", FheType::Uint128) => 88_000,
            ("min"|"max", FheType::Uint256) => 100_000,

            // comparisons — cost is based on input type; output is always ebool
            ("lt"|"le"|"gt"|"ge"|"eq"|"ne", FheType::Bool|FheType::Uint8) => 62_000,
            ("lt"|"le"|"gt"|"ge"|"eq"|"ne", FheType::Uint16) => 64_000,
            ("lt"|"le"|"gt"|"ge"|"eq"|"ne", FheType::Uint32) => 66_000,
            ("lt"|"le"|"gt"|"ge"|"eq"|"ne", FheType::Uint64) => 70_000,
            ("lt"|"le"|"gt"|"ge"|"eq"|"ne", FheType::Uint128)=> 82_000,
            ("lt"|"le"|"gt"|"ge"|"eq"|"ne", FheType::Uint256)=> 100_000,

            // select / cmux
            ("select", FheType::Uint128) => 100_000,
            ("select", FheType::Uint256) => 130_000,
            ("select", _)               => 90_000,

            // access control and decryption
            ("allow"|"allowThis", _)        => 0,
            ("requestDecryption", _)        => 200_000,
            ("makePubliclyDecryptable", _)  => 200_000,
            ("fromExternal", _)             => 50_000,
            ("isInitialized", _)            => 0,

            _ => 65_000,
        }
    };

    (on_chain, cop)
}
