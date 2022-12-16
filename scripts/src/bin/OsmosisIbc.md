# Osmosis - Juno IBC deployment flow

1. Deploy AnsHost contracts on both chains
2. Fill ans_host with contract pool addresses and transfer channels
3. Deploy Abstract infrastructure on Juno
4. Deploy Host and CW1 on Osmosis
5. Create channel between Client and Host
6. Check channel creation
7. Deploy an OS and enable the client
8. Try performing an IBC tx