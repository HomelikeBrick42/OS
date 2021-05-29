#include "./String.h"

extern "C" {
	static u8 StringFromUIntOutput[128]; // TODO: Allocate memory
	String StringFromUInt(u64 value) {
		u8 length = 0;

		u64 temp = value;
		while (temp / 10 > 0) {
			temp /= 10;
			length++;
		}

		u8 index = 0;
		while (value / 10 > 0) {
			u8 remainder = value % 10;
			value /= 10;
			StringFromUIntOutput[length - index] = remainder + '0';
			index++;
		}

		u8 remainder = value % 10;
		StringFromUIntOutput[length - index] = remainder + '0';

		return (String) {
			.Data = StringFromUIntOutput,
			.Length = cast(u64) length + 1,
		};
	}

	static u8 StringFromIntOutput[128]; // TODO: Allocate memory
	String StringFromInt(s64 value) {
		b8 isNegative = value < 0;
		if (isNegative) {
			value *= -1;
			StringFromIntOutput[0] = '-';
		}

		u8 length = 0;

		u64 temp = value;
		while (temp / 10 > 0) {
			temp /= 10;
			length++;
		}

		u8 index = 0;
		while (value / 10 > 0) {
			u8 remainder = value % 10;
			value /= 10;
			StringFromIntOutput[length - index + isNegative] = remainder + '0';
			index++;
		}

		u8 remainder = value % 10;
		StringFromIntOutput[length - index + isNegative] = remainder + '0';

		return (String) {
			.Data = StringFromIntOutput,
			.Length = cast(u64) length + 1 + isNegative,
		};
	}

	static u8 StringFromFloatOutput[128]; // TODO: Allocate memory
	String StringFromFloat(f64 value, u8 decimals) {
		String intString = StringFromInt(cast(s64) value);

		if (value < 0) {
			value *= -1;
		}
		value -= cast(s64) value;

		u64 length = intString.Length + decimals + (decimals == 0 ? 0 : 1);

		u64 index = 0;
		for (u64 i = 0; i < intString.Length; i++) {
			StringFromFloatOutput[i] = intString.Data[i];
			index++;
		}

		if (decimals > 0) {
			StringFromFloatOutput[index++] = '.';
			for (u64 i = 0; i < decimals; i++) {
				value *= 10;
				u8 remainder = cast(s64) value % 10;
				StringFromFloatOutput[index++] = remainder + '0';
			}
		}

		return (String) {
			.Data = StringFromFloatOutput,
			.Length = cast(u64) length,
		};
	}
}
