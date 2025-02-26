use solana_idlgen::idlgen;
idlgen!({
  "name": "auction",
  "version": "0.1.0",
  "metadata":{"address": "7VNBDULA3eH3ctDqx5ckpfZA1Xe2AkjUnGjuXe7de6bf"},
  "instructions": [
    {
      "name": "init_house",
      "accounts": [
        {
          "name": "admin",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "auction_house",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "system_program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args":[
        {
          "name": "fee",
          "type": "u16"
        },
        {
          "name": "name",
          "type": "string"
        }
      ]
    },
    {
      "name": "init_auction",
      "accounts":
      [
        {
          "name": "seller",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "auction_house",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "auction",
          "isMut": true,
          "isSigner": false
          },
        {
          "name": "mint_a",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "mint_b",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "seller_mint_a_ata",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "associated_token_program",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "system_program",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "token_program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "starting_price",
          "type": "u64"
        },
        {
          "name": "end",
          "type": "u64"
        },
        {
          "name": "amount",
          "type": "u64"
        },
        {
          "name": "decimal",
          "type": "u8"
        }
      ]
    },
    {
      "name": "bid",
      "accounts": [
        {
          "name": "bidder",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "mint_b",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "auction_house",
          "isMut": false,
          "isSigner": false
          },
        {
          "name": "auction",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bidder_mint_b_ata",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bid_state",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bidder_escrow",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "associated_token_program",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "token_program",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "system_program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": [
        {
          "name": "price",
          "type": "u64"
        },
        {
          "name": "decimal",
          "type": "u8"
        }
      ]
    },
    {
      "name": "finalize",
      "accounts": [
        {
          "name": "payer",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "seller",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bidder",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "admin",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "mint_a",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "mint_b",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "auction_house",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "auction",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bidder_mint_a_ata",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "seller_mint_b_ata",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "seller_mint_a_ata",
          "isMut": true,
          "isSigner": false

          },
        {
          "name": "house_mint_b_ata",
          "isMut": true,
          "isSigner": false
          },
        {
          "name": "bid_state",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bidder_escrow",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "vault",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "associated_token_program",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "token_program",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "system_program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
    },
    {
      "name": "withdraw",
      "accounts": [
        {
          "name": "bidder",
          "isMut": true,
          "isSigner": true
        },
        {
          "name": "mint_b",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "auction_house",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "auction",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "bidder_mint_b_ata",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bidder_escrow",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "bid_state",
          "isMut": true,
          "isSigner": false
        },
        {
          "name": "associated_token_program",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "token_program",
          "isMut": false,
          "isSigner": false
        },
        {
          "name": "system_program",
          "isMut": false,
          "isSigner": false
        }
      ],
      "args": []
      },
    {
      "name": "cancel",
      "accounts": [
          {
            "name": "seller",
            "isMut": true,
            "isSigner": true
          },
          {
            "name": "auction_house",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "auction",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "mint_a",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "mint_b",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "seller_mint_a_ata",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "vault",
            "isMut": true,
            "isSigner": false
          },
          {
            "name": "associated_token_program",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "system_program",
            "isMut": false,
            "isSigner": false
          },
          {
            "name": "token_program",
            "isMut": false,
            "isSigner": false
          }
        ],
        "args": []
    }
  ]
});
