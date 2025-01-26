# Onchain OMies Coin Flip

## Overview
This is a Mantra blockchain based coin flip gambling game where users can bet on heads or tails with a chance to double their money

## Key Features
- Bet on coin flip outcomes (Heads or Tails)
- On-chain randomization
- Automatic calculating and tracking user statistics
- Ability to claim rewards (unclaimed automatically in case of temporary app liquidity deficiency) when app balanced will be replenished

## How to Play

### Placing a Bet
1. Choose a side: Heads or Tails
2. Send a valid bet amount within the contract's specified minimum and maximum limits
3. Wait for the result

### Winning and Losing
- If you win, you'll receive double your original bet
- If you lose, your bet goes to the platform's balance
- A platform fee slightly decreases the winning probability

### Claiming Rewards
- If you have unclaimed winnings, use the Claim function to withdraw them

## Important Rules
- One flip per transaction
- Bet amount must be within contract-defined limits
- Only specified cryptocurrency denomination accepted (Om)

## User Statistics Tracked
- Total bets placed
- Total wins
- Return on Investment (ROI)
- Unclaimed rewards

## Admin Functions
- Deposit/withdraw platform funds (including queries to determine amount of available to withdraw revenue and liquidity to deposit)
- Update game configuration
- Pause/unpause game
- Transfer admin rights

## Risk Disclaimer
- Gambling involves financial risk
- Only bet what you can afford to lose
- Outcomes are randomized