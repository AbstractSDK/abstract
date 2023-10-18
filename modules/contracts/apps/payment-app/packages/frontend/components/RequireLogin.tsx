import { useChain } from '@cosmos-kit/react'
import { CHAIN_NAME } from '../config'
import { ReactNode } from 'react'
import { Text } from '@chakra-ui/react'

export const RequireLogin = ({ children }: { children: ReactNode }) => {
  const { isWalletConnected } = useChain(CHAIN_NAME)

  if (!isWalletConnected) {
    return <Text textAlign="center">Please connect your wallet to continue.</Text>
  }

  return <>{children}</>
}
