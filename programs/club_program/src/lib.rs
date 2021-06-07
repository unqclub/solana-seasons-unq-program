use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_option::COption;
use anchor_spl::token::{self, Mint, TokenAccount};

#[program]
pub mod club_program {
    use anchor_spl::token::{self, MintTo, Transfer};

    use super::*;

    /// Instruction for creating the new Club on the UNQ platform
    #[access_control(CreateClub::accounts(&ctx, nonce, &seeds))]
    pub fn create_club(
        ctx: Context<CreateClub>,
        nonce: u8,
        seeds: String,
    ) -> ProgramResult {
        msg!("Instruction: CreateClub");
        let seeds_sign = &[seeds.as_bytes(), &[nonce]];
        let signer = &[&seeds_sign[..]];

        msg!("CreateClub: Start filling club account");
        let club = &mut ctx.accounts.club;
        club.club_token_mint = *ctx.accounts.club_token_mint.to_account_info().key;
        club.curator = *ctx.accounts.curator.to_account_info().key;
        club.club_treasury = *ctx.accounts.club_treasury.to_account_info().key;
        club.club_token_account = *ctx.accounts.club_token_account.to_account_info().key;
        let mut club_vault = ctx.accounts.club_vault.load_init()?;
        club_vault.club_account = *club.to_account_info().key;
        club.club_vault = *ctx.accounts.club_vault.to_account_info().key;
        msg!("CreateClub: Club account set");

        msg!("CreateClub: Start minting ClubToken to clubTokenAccount");
        let cpi_mint_to_accounts = MintTo {
            mint: ctx.accounts.club_token_mint.to_account_info(),
            to: ctx.accounts.club_token_account.to_account_info(),
            authority: ctx.accounts.club_signer.to_account_info(),
        };
        let mut cpi_token_program = ctx.accounts.token_program.clone();
        let cpi_mint_to_ctx = CpiContext::new_with_signer(cpi_token_program, cpi_mint_to_accounts, 
            signer);


        token::mint_to(cpi_mint_to_ctx, 10)?;
        msg!("CreateClub: Minted 10 ClubTokens to clubTokenAccount");

        msg!("CreateClub: Start transfer of 10 ClubTokens to curatorClubTokenAccount");
        let cpi_transfer_accounts = Transfer {
            authority: ctx.accounts.club_signer.to_account_info(),
            from: ctx.accounts.club_token_account.to_account_info(),
            to: ctx.accounts.curator_club_token_account.to_account_info()
        };
        cpi_token_program = ctx.accounts.token_program.clone();
        let cpi_transfer_ctx = CpiContext::new_with_signer(cpi_token_program, cpi_transfer_accounts, signer);
        token::transfer(cpi_transfer_ctx, 10)?;
        msg!("CreateClub: Transfered 10 ClubTokens to curatorClubTokenAccount");

        // TODO: Mint FAKE ETH to curatorTreasury FAKE ETH account
        // msg!("ConfirmTrade: Start minting FAKE ETH to curatorTreasury FAKE ETH account");
        // //mint 12 ETH needed mint authority
        // let cpi_mint_fake_eth_accounts = MintTo {
        //     mint: ctx.accounts.fake_eth_mint.to_account_info(),
        //     to: ctx.accounts.fake_eth_initial_account.to_account_info(),
        //     authority: ctx.accounts.fake_eth_mint_authority.to_account_info(),
        // };
        // let cpi_token_program = ctx.accounts.token_program.clone();
        // let cpi_mint_fake_eth_ctx = CpiContext::new_with_signer(cpi_token_program, cpi_mint_fake_eth_accounts, 
        //     signer);
        // token::mint_to(cpi_mint_fake_eth_ctx, 5)?;
        // //transfer 12 ETH needed authority
        // let cpi_fake_eth_transfer_accounts = Transfer {
        //     authority: ctx.accounts.fake_eth_mint_authority.to_account_info(),
        //     from: ctx.accounts.fake_eth_initial_account.to_account_info(),
        //     to: ctx.accounts.club_treasury_fake_eth_account.to_account_info()
        // };
        // let cpi_token_program = ctx.accounts.token_program.clone();
        // let cpi_transfer_fake_eth_ctx = CpiContext::new_with_signer(cpi_token_program, cpi_fake_eth_transfer_accounts, signer);
        // token::transfer(cpi_transfer_fake_eth_ctx, 5)?;
        // msg!("ConfirmTrade: Minted 5 FAKE ETH to curatorTreasury FAKE ETH account");

        Ok(())
    }
    
    /// Instruction for initializing the new Trade on the UNQ platform
    pub fn initialize_trade(
        ctx: Context<InitializeTrade>, 
        nft_address: String, 
        trade_amount: u64) -> ProgramResult {
        msg!("Instruction: InitializeTrade");
            let nft_address_bytes = nft_address.as_bytes();
            let mut nft_address_holder = [0u8; 64];
            nft_address_holder[..nft_address_bytes.len()].copy_from_slice(nft_address_bytes);

        let trade = &mut ctx.accounts.trade;
        trade.nft_address = nft_address_holder;
        trade.trade_amount = trade_amount;
        trade.club = *ctx.accounts.club.to_account_info().key;
        // TODO: add chain_id and token_id
        // trade.chain_id = "ethereum".to_string();
        // trade.token_id = token_id;
        msg!("InitializeTrade: Initialized trade");

        Ok(())
    }

    /// Instruction for confirming the Trade on the UNQ platform
    pub fn confirm_trade(
        ctx: Context<ConfirmTrade>,
        tx_id: String, 
    ) -> ProgramResult {
        msg!("Instrukcija: ConfirmTrade");


        let tx_id_bytes = tx_id.as_bytes();
            let mut tx_id_holder = [0u8; 128];
            tx_id_holder[..tx_id_bytes.len()].copy_from_slice(tx_id_bytes);
        msg!("ConfirmTrade: Converted tx string");

        let trade = &mut ctx.accounts.trade;
        trade.chain_tx_id = tx_id_holder;
        msg!("ConfirmTrade: Updated trade with tx id");

        msg!("ConfirmTrade: Start putting NFT to club vault");
        let mut club_vault = ctx.accounts.club_vault.load_mut()?;
        let nft_entry = NFTOwnership {
            nft_address: trade.nft_address,
            tx_id: tx_id_holder
        };
        club_vault.append(nft_entry);

        msg!("ConfirmTrade: NFT added to the club vault");


        Ok(())
    }
}

#[derive(Accounts)]
pub struct CreateClub<'info> {
    // New Club account being created.
    #[account(init)]
    pub club: ProgramAccount<'info, ClubAccount>,
    // Program derived address for the Club.
    pub club_signer: AccountInfo<'info>,
    // Club token mint account
    #[account(mut, "club_token_mint.mint_authority == COption::Some(*club_signer.key)")]
    pub club_token_mint: CpiAccount<'info, Mint>,
    // Club token account
    #[account(mut, "club_token_account.owner == *club_signer.key")]
    pub club_token_account: CpiAccount<'info, TokenAccount>,
    // Club vault pubkey
    #[account(init)]
    pub club_vault: Loader<'info, ClubVault>,
    // Club treasury pubkey
    #[account(init)]
    pub club_treasury: ProgramAccount<'info, ClubTreasury>,
    // // Fake ETH account in club treasury
    // #[account(mut, "club_treasury_fake_eth_account.owner == *club_signer.key")]
    // pub club_treasury_fake_eth_account: CpiAccount<'info, TokenAccount>,
    // // Fake ETH Mint account - EFj4M47qHDRtDKhrP5ZJqbYtpJ3VYWyiBFf9hn9AYud
    // #[account(mut, "fake_eth_mint.mint_authority == COption::Some(*fake_eth_mint_authority.key)")]
    // pub fake_eth_mint: CpiAccount<'info, Mint>,
    // // Fake ETH initial account - LjBE1ZXemU1nXWdEnHD8k1YwEUqs1jxgN1xGxUsnyXh
    // #[account(mut, "fake_eth_initial_account.owner == *fake_eth_mint_authority.key")]
    // pub fake_eth_initial_account: CpiAccount<'info, TokenAccount>,
    // // Fake ETH Mint authority - Bi8bbtxe64xtgEs5LXhu2jykVBU6uhUJGqbgZEq2FSUq
    // pub fake_eth_mint_authority: AccountInfo<'info>,
    // Curator pubkey
    #[account(signer)]
    pub curator: AccountInfo<'info>,
    #[account(mut, "curator_club_token_account.owner == *curator.key")]
    pub curator_club_token_account: CpiAccount<'info, TokenAccount>,
    #[account("token_program.key == &token::ID")]
    pub token_program: AccountInfo<'info>,
    pub rent: Sysvar<'info, Rent>
}

impl<'info> CreateClub<'info> {
    fn accounts(ctx: &Context<CreateClub<'info>>, nonce: u8, seeds: &str) -> Result<()> {
        msg!("NONCE: {}", nonce);
        msg!("String: {}", seeds);
        let expected_signer = Pubkey::create_program_address(
            &[seeds.as_bytes(), &[nonce]], ctx.program_id)
            .map_err(|_| ErrorCode::InvalidNonce)?;
            if ctx.accounts.club_signer.key != &expected_signer {
                return Err(ErrorCode::InvalidNonce.into());
            }
            Ok(())
    }
}

#[derive(Accounts)]
pub struct InitializeTrade<'info> {
    // Trade being created
    #[account(init)]
    pub trade: ProgramAccount<'info, Trade>,
    // Club initializing the trade
    pub club: ProgramAccount<'info, ClubAccount>,
    // Program derived address for the trade.
    pub trade_signer: AccountInfo<'info>,
    // trade_token_mint_account: AccountInfo<'info>,
    // #[account(mut, "trade_token_account.owner == *trade_signer.key")]
    trade_token_account: CpiAccount<'info, TokenAccount>,
    #[account("token_program.key == &token::ID")]
    token_program: AccountInfo<'info>,
    rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct ConfirmTrade<'info> {
    // Trade being confirmed
    #[account(mut)]
    pub trade: ProgramAccount<'info, Trade>,
    // Club vault account
    #[account(mut)]
    pub club_vault: Loader<'info, ClubVault>
}


#[account(zero_copy)]
pub struct ClubVault {
    // Head and tail of the ClubVault list
    head: u64,
    tail: u64,
    /// Account of Club owning the vault
    pub club_account: Pubkey,
    /// Array of NFTs
    pub nfts: [NFTOwnership; 150]
}
// Implement append in order to add NFT ownership
impl ClubVault {
    fn append(&mut self, nft_ownership: NFTOwnership) {
        self.nfts[ClubVault::index_of(self.head)] = nft_ownership;
        if ClubVault::index_of(self.head + 1) == ClubVault::index_of(self.tail) {
            self.tail += 1;
        }
        self.head += 1;
    }
    fn index_of(counter: u64) -> usize {
        std::convert::TryInto::try_into(counter % 33607).unwrap()
    }
}

#[zero_copy]
pub struct NFTOwnership {
    pub nft_address: [u8; 64],
    // TODO: Add nft_token_id and chain_id
    // pub nft_token_id: u64,
    // pub chain_id: u8,
    pub tx_id: [u8; 128]
}

#[account]
pub struct ClubAccount {
    pub curator: Pubkey,
    pub club_token_mint: Pubkey,
    pub club_token_account: Pubkey,
    pub club_treasury: Pubkey,
    pub club_vault: Pubkey
}

#[account]
pub struct ClubTreasury {
    pub club_account: Pubkey,
    pub treasury_mint: Pubkey,
    pub treasury_amount: u64
    // TODO: Add club treasury entry
    // pub club_treasury_entry: Vec<Pubkey, u64>
}

#[account]
pub struct Trade {
 pub nft_address: [u8; 64],
 pub nft_token_id: u8,
 pub trade_amount: u64,
 pub club: Pubkey,
 // TODO: add chain id
 //  pub chain_id: String,
 pub chain_tx_id: [u8; 128],
}

#[error]
pub enum ErrorCode {
    #[msg("The given trade signer does not create a valid program derived address.")]
    InvalidTradeSigner,
    #[msg("The given nonce is not valid")]
    InvalidNonce,
}