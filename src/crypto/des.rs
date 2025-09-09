//! Self-implemented DES, use to make dependencies minimize,
//! and the performance gap is negligible for the small amount of data we pass on

// 唯一的用处就是在 ClassApi 加密一段 URL, 干脆自己实现一个算了
pub struct Des {
    subkeys: [u64; 16],
}

impl Des {
    /// Create a DES instance with the given key
    pub fn new(key: &[u8]) -> Result<Self, &'static str> {
        if key.len() != 8 {
            return Err("DES key must be 8 bytes");
        }

        let mut des = Des { subkeys: [0; 16] };
        des.generate_subkeys(key);
        Ok(des)
    }

    /// Generate a 16-wheel subkey
    fn generate_subkeys(&mut self, key: &[u8]) {
        // PC-1 置换表 (56位)

        // 前导零仅为了排列美观, 包括下面禁用格式化一样
        // 不过在类 C 语言中前导零表示八进制
        // 而 Rust 没有这个问题, 所以禁用警告
        #[allow(clippy::zero_prefixed_literal)]
        #[rustfmt::skip]
        const PC1: [usize; 56] = [
            57, 49, 41, 33, 25, 17, 09,
            01, 58, 50, 42, 34, 26, 18,
            10, 02, 59, 51, 43, 35, 27,
            19, 11, 03, 60, 52, 44, 36,
            63, 55, 47, 39, 31, 23, 15,
            07, 62, 54, 46, 38, 30, 22,
            14, 06, 61, 53, 45, 37, 29,
            21, 13, 05, 28, 20, 12, 04,
        ];

        // PC-2 置换表 (48位)

        // 前导零仅为了排列美观, 包括下面禁用格式化一样
        // 不过在类 C 语言中前导零表示八进制
        // 而 Rust 没有这个问题, 所以禁用警告
        #[allow(clippy::zero_prefixed_literal)]
        #[rustfmt::skip]
        const PC2: [usize; 48] = [
            14, 17, 11, 24, 01, 05,
            03, 28, 15, 06, 21, 10,
            23, 19, 12, 04, 26, 08,
            16, 07, 27, 20, 13, 02,
            41, 52, 31, 37, 47, 55,
            30, 40, 51, 45, 33, 48,
            44, 49, 39, 56, 34, 53,
            46, 42, 50, 36, 29, 32,
        ];

        // 左循环移位表
        const SHIFTS: [usize; 16] = [1, 1, 2, 2, 2, 2, 2, 2, 1, 2, 2, 2, 2, 2, 2, 1];

        // 将密钥转换为 64 位
        let key_bits = u64::from_be_bytes(key.try_into().unwrap());

        // 应用 PC-1 置换
        let mut pc1_result = 0u64;
        for (i, &pos) in PC1.iter().enumerate() {
            let bit = (key_bits >> (64 - pos)) & 1;
            pc1_result |= bit << (55 - i);
        }

        // 分割成左右 28 位
        let mut left = (pc1_result >> 28) & 0x0FFFFFFF;
        let mut right = pc1_result & 0x0FFFFFFF;

        // 生成 16 轮子密钥
        for (round, &shift) in SHIFTS.iter().enumerate() {
            // 左循环移位
            left = ((left << shift) | (left >> (28 - shift))) & 0x0FFFFFFF;
            right = ((right << shift) | (right >> (28 - shift))) & 0x0FFFFFFF;

            // 合并并应用 PC-2 置换
            let combined = (left << 28) | right;
            let mut subkey = 0u64;

            for (i, &pos) in PC2.iter().enumerate() {
                let bit_pos = 56 - pos; // PC-2 表是基于 56 位输入的
                let bit = (combined >> bit_pos) & 1;
                subkey |= bit << (47 - i);
            }

            self.subkeys[round] = subkey;
        }
    }

    /// S-box replacement
    fn s_box(&self, box_num: usize, input: u8) -> u8 {
        // S 盒已经对其, 不再添加前导零
        #[rustfmt::skip]
        const S_BOXES: [[[u8; 16]; 4]; 8] = [
            // S1
            [
                [14, 4, 13, 1, 2, 15, 11, 8, 3, 10, 6, 12, 5, 9, 0, 7],
                [0, 15, 7, 4, 14, 2, 13, 1, 10, 6, 12, 11, 9, 5, 3, 8],
                [4, 1, 14, 8, 13, 6, 2, 11, 15, 12, 9, 7, 3, 10, 5, 0],
                [15, 12, 8, 2, 4, 9, 1, 7, 5, 11, 3, 14, 10, 0, 6, 13],
            ],
            // S2
            [
                [15, 1, 8, 14, 6, 11, 3, 4, 9, 7, 2, 13, 12, 0, 5, 10],
                [3, 13, 4, 7, 15, 2, 8, 14, 12, 0, 1, 10, 6, 9, 11, 5],
                [0, 14, 7, 11, 10, 4, 13, 1, 5, 8, 12, 6, 9, 3, 2, 15],
                [13, 8, 10, 1, 3, 15, 4, 2, 11, 6, 7, 12, 0, 5, 14, 9],
            ],
            // S3
            [
                [10, 0, 9, 14, 6, 3, 15, 5, 1, 13, 12, 7, 11, 4, 2, 8],
                [13, 7, 0, 9, 3, 4, 6, 10, 2, 8, 5, 14, 12, 11, 15, 1],
                [13, 6, 4, 9, 8, 15, 3, 0, 11, 1, 2, 12, 5, 10, 14, 7],
                [1, 10, 13, 0, 6, 9, 8, 7, 4, 15, 14, 3, 11, 5, 2, 12],
            ],
            // S4
            [
                [7, 13, 14, 3, 0, 6, 9, 10, 1, 2, 8, 5, 11, 12, 4, 15],
                [13, 8, 11, 5, 6, 15, 0, 3, 4, 7, 2, 12, 1, 10, 14, 9],
                [10, 6, 9, 0, 12, 11, 7, 13, 15, 1, 3, 14, 5, 2, 8, 4],
                [3, 15, 0, 6, 10, 1, 13, 8, 9, 4, 5, 11, 12, 7, 2, 14],
            ],
            // S5
            [
                [2, 12, 4, 1, 7, 10, 11, 6, 8, 5, 3, 15, 13, 0, 14, 9],
                [14, 11, 2, 12, 4, 7, 13, 1, 5, 0, 15, 10, 3, 9, 8, 6],
                [4, 2, 1, 11, 10, 13, 7, 8, 15, 9, 12, 5, 6, 3, 0, 14],
                [11, 8, 12, 7, 1, 14, 2, 13, 6, 15, 0, 9, 10, 4, 5, 3],
            ],
            // S6
            [
                [12, 1, 10, 15, 9, 2, 6, 8, 0, 13, 3, 4, 14, 7, 5, 11],
                [10, 15, 4, 2, 7, 12, 9, 5, 6, 1, 13, 14, 0, 11, 3, 8],
                [9, 14, 15, 5, 2, 8, 12, 3, 7, 0, 4, 10, 1, 13, 11, 6],
                [4, 3, 2, 12, 9, 5, 15, 10, 11, 14, 1, 7, 6, 0, 8, 13],
            ],
            // S7
            [
                [4, 11, 2, 14, 15, 0, 8, 13, 3, 12, 9, 7, 5, 10, 6, 1],
                [13, 0, 11, 7, 4, 9, 1, 10, 14, 3, 5, 12, 2, 15, 8, 6],
                [1, 4, 11, 13, 12, 3, 7, 14, 10, 15, 6, 8, 0, 5, 9, 2],
                [6, 11, 13, 8, 1, 4, 10, 7, 9, 5, 0, 15, 14, 2, 3, 12],
            ],
            // S8
            [
                [13, 2, 8, 4, 6, 15, 11, 1, 10, 9, 3, 14, 5, 0, 12, 7],
                [1, 15, 13, 8, 10, 3, 7, 4, 12, 5, 6, 11, 0, 14, 9, 2],
                [7, 11, 4, 1, 9, 12, 14, 2, 0, 6, 10, 13, 15, 3, 5, 8],
                [2, 1, 14, 7, 4, 10, 8, 13, 15, 12, 9, 0, 3, 5, 6, 11],
            ],
        ];

        let row = ((input & 0x20) >> 4 | (input & 1)) as usize;
        let col = ((input >> 1) & 0x0F) as usize;

        S_BOXES[box_num][row][col]
    }

    /// Feistel 函数
    fn feistel(&self, right: u32, subkey: u64) -> u32 {
        // 扩展置换表 E

        // 前导零仅为了排列美观, 包括下面禁用格式化一样
        // 不过在类 C 语言中前导零表示八进制
        // 而 Rust 没有这个问题, 所以禁用警告
        #[allow(clippy::zero_prefixed_literal)]
        #[rustfmt::skip]
        const E: [usize; 48] = [
            32, 01, 02, 03, 04, 05,
            04, 05, 06, 07, 08, 09,
            08, 09, 10, 11, 12, 13,
            12, 13, 14, 15, 16, 17,
            16, 17, 18, 19, 20, 21,
            20, 21, 22, 23, 24, 25,
            24, 25, 26, 27, 28, 29,
            28, 29, 30, 31, 32, 01,
        ];

        // P 置换表

        // 前导零仅为了排列美观, 包括下面禁用格式化一样
        // 不过在类 C 语言中前导零表示八进制
        // 而 Rust 没有这个问题, 所以禁用警告
        #[allow(clippy::zero_prefixed_literal)]
        #[rustfmt::skip]
        const P: [usize; 32] = [
            16, 07, 20, 21,
            29, 12, 28, 17,
            01, 15, 23, 26,
            05, 18, 31, 10,
            02, 08, 24, 14,
            32, 27, 03, 09,
            19, 13, 30, 06,
            22, 11, 04, 25,
        ];

        // 扩展置换
        let mut expanded = 0u64;
        for (i, &pos) in E.iter().enumerate() {
            let bit = (right >> (32 - pos)) & 1;
            expanded |= (bit as u64) << (47 - i);
        }

        // 与子密钥异或
        let xor_result = expanded ^ subkey;

        // S 盒替换
        let mut sbox_output = 0u32;
        for i in 0..8 {
            let chunk = ((xor_result >> (42 - i * 6)) & 0x3F) as u8;
            let sbox_val = self.s_box(i, chunk);
            sbox_output = (sbox_output << 4) | sbox_val as u32;
        }

        // P 置换
        let mut permuted = 0u32;
        for (i, &pos) in P.iter().enumerate() {
            let bit = (sbox_output >> (32 - pos)) & 1;
            permuted |= bit << (31 - i);
        }

        permuted
    }

    /// Encrypt a single 8-byte block
    pub fn encrypt_block(&self, block: &[u8]) -> [u8; 8] {
        // 初始置换表 IP

        // 前导零仅为了排列美观, 包括下面禁用格式化一样
        // 不过在类 C 语言中前导零表示八进制
        // 而 Rust 没有这个问题, 所以禁用警告
        #[allow(clippy::zero_prefixed_literal)]
        #[rustfmt::skip]
        const IP: [usize; 64] = [
            58, 50, 42, 34, 26, 18, 10, 02,
            60, 52, 44, 36, 28, 20, 12, 04,
            62, 54, 46, 38, 30, 22, 14, 06,
            64, 56, 48, 40, 32, 24, 16, 08,
            57, 49, 41, 33, 25, 17, 09, 01,
            59, 51, 43, 35, 27, 19, 11, 03,
            61, 53, 45, 37, 29, 21, 13, 05,
            63, 55, 47, 39, 31, 23, 15, 07,
        ];

        // 最终置换表 IP^-1

        // 前导零仅为了排列美观, 包括下面禁用格式化一样
        // 不过在类 C 语言中前导零表示八进制
        // 而 Rust 没有这个问题, 所以禁用警告
        #[allow(clippy::zero_prefixed_literal)]
        #[rustfmt::skip]
        const FP: [usize; 64] = [
            40, 08, 48, 16, 56, 24, 64, 32,
            39, 07, 47, 15, 55, 23, 63, 31,
            38, 06, 46, 14, 54, 22, 62, 30,
            37, 05, 45, 13, 53, 21, 61, 29,
            36, 04, 44, 12, 52, 20, 60, 28,
            35, 03, 43, 11, 51, 19, 59, 27,
            34, 02, 42, 10, 50, 18, 58, 26,
            33, 01, 41, 09, 49, 17, 57, 25,
        ];

        // 将块转换为 64 位
        let block_bits = u64::from_be_bytes(block.try_into().unwrap());

        // 应用初始置换 IP
        let mut data = 0u64;
        for (i, &pos) in IP.iter().enumerate() {
            let bit = (block_bits >> (64 - pos)) & 1;
            data |= bit << (63 - i);
        }

        // 分割成左右 32 位
        let mut left = (data >> 32) as u32;
        let mut right = data as u32;

        // 16 轮 Feistel 网络
        for i in 0..16 {
            let temp = right;
            right = left ^ self.feistel(right, self.subkeys[i]);
            left = temp;
        }

        // 合并左右部分 (注意: 最后一轮后不交换)
        let combined = ((right as u64) << 32) | (left as u64);

        // 应用最终置换 FP
        let mut output = 0u64;
        for (i, &pos) in FP.iter().enumerate() {
            let bit = (combined >> (64 - pos)) & 1;
            output |= bit << (63 - i);
        }

        output.to_be_bytes()
    }

    /// DES encrypt, use ECB mode, PKCS5Padding
    pub fn encrypt_ecb(&self, data: &[u8]) -> Vec<u8> {
        let mut input = data.to_vec();

        // PKCS5Padding 填充
        let padding_len = 8 - input.len() % 8;
        input.extend(vec![padding_len as u8; padding_len]);

        // 创建输出缓冲区
        let mut output = vec![0u8; input.len()];

        // 加密每个块
        for (i, chunk) in input.chunks(8).enumerate() {
            let block = self.encrypt_block(chunk);
            output[i * 8..(i + 1) * 8].copy_from_slice(&block);
        }

        output
    }
}
