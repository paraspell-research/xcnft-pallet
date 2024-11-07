/// Code originates from pallet_nfts
///
/// Used to mimic CollectionConfigFor parameter from NFTs pallet
macro_rules! impl_codec_bitflags {
	($wrapper:ty, $size:ty, $bitflag_enum:ty) => {
		impl MaxEncodedLen for $wrapper {
			fn max_encoded_len() -> usize {
				<$size>::max_encoded_len()
			}
		}
		impl Encode for $wrapper {
			fn using_encoded<R, F: FnOnce(&[u8]) -> R>(&self, f: F) -> R {
				self.0.bits().using_encoded(f)
			}
		}
		impl EncodeLike for $wrapper {}
		impl Decode for $wrapper {
			fn decode<I: codec::Input>(
				input: &mut I,
			) -> ::core::result::Result<Self, codec::Error> {
				let field = <$size>::decode(input)?;
				Ok(Self(BitFlags::from_bits(field as $size).map_err(|_| "invalid value")?))
			}
		}

		impl TypeInfo for $wrapper {
			type Identity = Self;

			fn type_info() -> Type {
				Type::builder()
					.path(Path::new("BitFlags", module_path!()))
					.type_params(vec![TypeParameter::new("T", Some(meta_type::<$bitflag_enum>()))])
					.composite(
						Fields::unnamed()
							.field(|f| f.ty::<$size>().type_name(stringify!($bitflag_enum))),
					)
			}
		}
	};
}
pub(crate) use impl_codec_bitflags;
