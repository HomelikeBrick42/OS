#include "./String.h"

extern "C" {
	String StringFromu8(u8 value) {
		return StringFromu64(cast(u64) value);
	}

	String StringFromu16(u16 value) {
		return StringFromu64(cast(u64) value);
	}

	String StringFromu32(u32 value) {
		return StringFromu64(cast(u64) value);
	}

	static u8 StringFromu64Output[128]; // TODO: Allocate memory
	String StringFromu64(u64 value) {
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
			StringFromu64Output[length - index] = remainder + '0';
			index++;
		}

		u8 remainder = value % 10;
		StringFromu64Output[length - index] = remainder + '0';

		return (String) {
			.Data = StringFromu64Output,
			.Length = cast(u64) length + 1,
		};
	}

	String StringFroms8(s8 value) {
		return StringFroms64(cast(s64) value);
	}
	
	String StringFroms16(s16 value) {
		return StringFroms64(cast(s64) value);
	}

	String StringFroms32(s32 value) {
		return StringFroms64(cast(s64) value);
	}

	static u8 StringFroms64Output[128]; // TODO: Allocate memory
	String StringFroms64(s64 value) {
		b8 isNegative = value < 0;
		if (isNegative) {
			value *= -1;
			StringFroms64Output[0] = '-';
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
			StringFroms64Output[length - index + isNegative] = remainder + '0';
			index++;
		}

		u8 remainder = value % 10;
		StringFroms64Output[length - index + isNegative] = remainder + '0';

		return (String) {
			.Data = StringFroms64Output,
			.Length = cast(u64) length + 1 + isNegative,
		};
	}

	String StringFromf32(f32 value, u8 decimals) {
		return StringFromf64(cast(f64) value, decimals);
	}

	static u8 StringFromf64Output[128]; // TODO: Allocate memory
	String StringFromf64(f64 value, u8 decimals) {
		String intString = StringFroms64(cast(s64) value);

		if (value < 0) {
			value *= -1;
		}
		value -= cast(s64) value;

		u64 length = intString.Length + decimals + (decimals == 0 ? 0 : 1);

		u64 index = 0;
		for (u64 i = 0; i < intString.Length; i++) {
			StringFromf64Output[i] = intString.Data[i];
			index++;
		}

		if (decimals > 0) {
			StringFromf64Output[index++] = '.';
			for (u64 i = 0; i < decimals; i++) {
				value *= 10;
				u8 remainder = cast(s64) value % 10;
				StringFromf64Output[index++] = remainder + '0';
			}
		}

		return (String) {
			.Data = StringFromf64Output,
			.Length = cast(u64) length,
		};
	}
}
