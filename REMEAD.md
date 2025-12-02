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
## JSON解析
交易的 JSON 解析结构与常规 JSON 格式类似，但增加了账户和指令信息的解析：
"transaction": {
  "message": {
    "accountKeys": [
      {
        "pubkey": "EF3cbGuhJus5mCdGZkVz7GQce7QHbswBhZu6fmK9zkCR",
        "signer": true,
        "source": "transaction",
        "writable": true
      },
      {
        "pubkey": "4LAyP5B5jNyNm7Ar2dG8sNipEiwTMEyCHd1iCHhhXYkY",
        "signer": false,
        "source": "transaction",
        "writable": true
      },
      {
        "pubkey": "11111111111111111111111111111111",
        "signer": false,
        "source": "transaction",
        "writable": false
      }
    ],
    "instructions": [
      {
        "parsed": {
          "info": {
            "destination": "4LAyP5B5jNyNm7Ar2dG8sNipEiwTMEyCHd1iCHhhXYkY",
            "lamports": 100000000,
            "source": "EF3cbGuhJus5mCdGZkVz7GQce7QHbswBhZu6fmK9zkCR"
          },
          "type": "transfer"
        },
        "program": "system",
        "programId": "11111111111111111111111111111111",
        "stackHeight": null
      }
    ],
    "recentBlockhash": "6pw7JBwq9tb5GHiBQgVY6RAp5otbouwYvEc1kbbxKFec"
  },
  "signatures": [
    "2M8mvwhtxyz3vAokXESVeR9FQ4t9QQxF5ek6ENNBBHVkW5XyZvJVK5MQej5ccwTZH6iWBJJoZ2CcizBs89pvpPBh"
  ]
}
message: <object> - 定义交易的内容。
accountKeys: <array[object]> - 交易使用的账户信息列表。
pubkey: <string> - 账户的 base-58 编码公钥。
signer: <boolean> - 指示账户是否为必需的交易签名者。
writable: <boolean> - 指示账户是否可写。
source: <string> - 账户的来源（交易或查找表）。
recentBlockhash: <string> - 用于防止交易重复并为交易设定生命周期的账本中最近区块的 base-58 编码哈希。
instructions: <array[object]> - 已解析的程序指令列表。
program: <string> - 被调用程序的名称。
programId: <string> - 程序的 base-58 编码公钥。
stackHeight: <number|null> - 指令的堆栈高度。
parsed: <object> - 程序特定的已解析数据。
type: <string> - 指令的类型（例如，“转账”）。
info: <object> - 特定于程序和指令类型的已解析指令信息。
signatures: <array[string]> - 应用于交易的 base-58 编码签名列表。
### 交易状态元数据
{
  "meta": {
    "err": null,
    "fee": 5000,
    "innerInstructions": [],
    "logMessages": [],
    "postBalances": [499998932500, 26858640, 1, 1, 1],
    "postTokenBalances": [],
    "preBalances": [499998937500, 26858640, 1, 1, 1],
    "preTokenBalances": [],
    "rewards": null,
    "status": {
      "Ok": null
    }
  }
}
err: <object|null> - 如果交易失败则显示错误信息，如果交易成功则为 null。 TransactionError 定义
fee: <u64> - 此交易收取的费用，以 u64 整数表示
preBalances: <array> - 交易处理前的 u64 账户余额数组
postBalances: <array> - 交易处理后的 u64 账户余额数组
innerInstructions: <array|null> - 内部指令列表，或 null 如果在此交易期间未启用内部指令记录
preTokenBalances: <array|undefined> - 代币余额列表，显示交易处理前的代币余额，或如果在此交易期间尚未启用代币余额记录则省略
postTokenBalances: <array|undefined> - 代币余额列表，显示交易处理后的代币余额，或如果在此交易期间尚未启用代币余额记录则省略
logMessages: <array|null> - 字符串日志消息数组，或 null 如果在此交易期间未启用日志消息记录
rewards: <array|null> - 交易级别的奖励；包含以下字段的 JSON 对象数组：
pubkey: <string> - 接收奖励的账户的 base-58 编码公钥字符串
lamports: <i64> - 账户获得或扣除的奖励 lamports 数量，以 i64 表示
postBalance: <u64> - 应用奖励后账户的 lamports 余额
rewardType: <string|undefined> - 奖励类型：“fee”、“rent”、“voting”、“staking”
commission: <u8|undefined> - 奖励记入时的投票账户佣金，仅适用于投票和质押奖励
已弃用：status: <object> - 交易状态
"Ok": <null> - 交易成功
"Err": <ERR> - 交易因 TransactionError 失败
loadedAddresses: <object|undefined> - 从地址查找表加载的交易地址。如果 maxSupportedTransactionVersion 未在请求参数中设置，或 jsonParsed 编码在请求参数中设置，则未定义。
writable: <array[string]> - 可写加载账户的 base-58 编码地址的有序列表
readonly: <array[string]> - 只读加载账户的 base-58 编码地址的有序列表
returnData: <object|undefined> - 交易中指令生成的最新返回数据，包含以下字段：
programId: <string> - 生成返回数据的程序，作为 base-58 编码的 Pubkey
data: <[string, encoding]> - 返回数据本身，作为 base-64 编码的二进制数据
computeUnitsConsumed: <u64|undefined> - 交易消耗的 计算单元数量
version: <"legacy"|number|undefined> - 交易版本。如果 maxSupportedTransactionVersion 未在请求参数中设置，则未定义。
signatures: <array> - 如果请求了交易详细信息的“签名”，则显示；一个签名字符串数组，对应区块中的交易顺序