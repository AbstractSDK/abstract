import { useChain } from '@cosmos-kit/react'
import { CHAIN_NAME } from '../../config'
import { WalletStatus } from '@cosmos-kit/core'
import { ConnectWalletButton } from './ConnectWalletButton'
import { Error } from './Error'

export const WalletButton = () => {
  const { connect, openView, status } = useChain(CHAIN_NAME)

  switch (status) {
    case WalletStatus.Connecting:
      return <ConnectWalletButton isLoading />
    case WalletStatus.Connected:
      return <ConnectWalletButton buttonText="My Wallet" onClick={() => openView()} />
    case WalletStatus.Rejected:
      return <Error buttonText="Reconnect" onClick={() => connect()} />
    case WalletStatus.Error:
      return <Error buttonText="Change Wallet" onClick={openView} />
    case WalletStatus.NotExist:
      return <ConnectWalletButton buttonText="Install Wallet" onClick={() => openView()} />
    // WalletStatus.Disconnected
    default:
      return <ConnectWalletButton buttonText="Connect Wallet" onClick={() => connect()} />
  }
}
