## Solana RPC 方法的常见 JSON 数据结构
### 交易的JSON结构定义如下：
"""
"transaction": {
  "message": {
    "accountKeys": [
      "EF3cbGuhJus5mCdGZkVz7GQce7QHbswBhZu6fmK9zkCR",
      "4LAyP5B5jNyNm7Ar2dG8sNipEiwTMEyCHd1iCHhhXYkY",
      "11111111111111111111111111111111"
    ],
    "header": {
      "numReadonlySignedAccounts": 0,
      "numReadonlyUnsignedAccounts": 1,
      "numRequiredSignatures": 1
    },
    "instructions": [
      {
        "accounts": [
          0,
          1
        ],
        "data": "3Bxs411Dtc7pkFQj",
        "programIdIndex": 2,
        "stackHeight": null
      }
    ],
    "recentBlockhash": "6pw7JBwq9tb5GHiBQgVY6RAp5otbouwYvEc1kbbxKFec"
  },
  "signatures": [
    "2M8mvwhtxyz3vAokXESVeR9FQ4t9QQxF5ek6ENNBBHVkW5XyZvJVK5MQej5ccwTZH6iWBJJoZ2CcizBs89pvpPBh"
  ]
}
"""
message: <object> - 定义交易的内容。
accountKeys: <array[string]> - 交易中使用的 base-58 编码公钥列表，包括指令和签名所需的公钥。第一个 message.header.numRequiredSignatures 公钥必须对交易进行签名。
header: <object> - 详细说明交易所需的账户类型和签名。
numRequiredSignatures: <number> - 使交易有效所需的签名总数。这些签名必须与 numRequiredSignatures 的第一个 message.accountKeys 匹配。
numReadonlySignedAccounts: <number> - 签名密钥的最后 numReadonlySignedAccounts 是只读账户。程序可以在单个 PoH 条目中处理加载只读账户的多个交易，但不允许对 lamports 进行借记或贷记，也不能修改账户数据。目标为相同读写账户的交易将按顺序评估。
numReadonlyUnsignedAccounts: <number> - 未签名密钥的最后 numReadonlyUnsignedAccounts 是只读账户。
recentBlockhash: <string> - 最近区块的 base-58 编码哈希，用于防止交易重复并为交易提供生命周期。
instructions: <array[object]> - 按顺序执行并在全部成功时以原子交易提交的程序指令列表。
programIdIndex: <number> - 指向 message.accountKeys 数组中执行此指令的程序账户的索引。
accounts: <array[number]> - 指向 message.accountKeys 数组中传递给程序的账户的有序索引列表。
data: <string> - 以 base-58 字符串编码的程序输入数据。
addressTableLookups: <array[object]|undefined> - 交易使用的地址表查找列表，用于从链上地址查找表动态加载地址。如果 maxSupportedTransactionVersion 未设置，则未定义。
accountKey: <string> - 地址查找表账户的 base-58 编码公钥。
writableIndexes: <array[number]> - 用于从查找表加载可写账户地址的索引列表。
readonlyIndexes: <array[number]> - 用于从查找表加载只读账户地址的索引列表。
signatures: <array[string]> - 应用于交易的 base-58 编码签名列表。列表的长度始终为 message.header.numRequiredSignatures 且不为空。索引 i 处的签名对应于 message.accountKeys 中索引 i 处的公钥。第一个签名用作 交易 ID。