import { Coin } from '@cosmjs/amino'
import { useChain } from '@cosmos-kit/react-lite'
import { useState, useEffect } from 'react'
import { CHAIN_NAME } from '../../config'
import { useAccount } from '../../contexts'
import {
  Box,
  Stack,
  Text,
} from '@chakra-ui/react'

export const AccountBalances = () => {
  const { getStargateClient } = useChain(CHAIN_NAME)
  const { getSigningAccountClient } = useAccount()

  const [balances, setBalances] = useState<readonly Coin[]>()
  useEffect(() => {
    ;(async () => {
      const accountClient = await getSigningAccountClient()
      const stargateClient = await getStargateClient()

      const balances = await stargateClient.getAllBalances(accountClient.proxyAddress)
      setBalances(balances)
    })()
  }, [getSigningAccountClient, getStargateClient])

  if (!balances) {
    return <Text textAlign="center">Loading...</Text>
  }

  return (
    <Stack
      p={5}
      w="full"
      maxW="xl"
      mx="auto"
      gap="1rem"
      boxShadow="0 0 2px #ccc, 0 0 5px -1px #ccc"
      borderRadius="md"
    >
      <Text>Balances</Text>

      <Stack>
        {balances.map(({ amount, denom }) => (
          <Box
            key={denom}
            p="3rem"
            boxShadow="0 0 2px #ccc, 0 0 5px -1px #ccc"
            borderRadius="md"
            display="flex"
            justifyContent="center"
            alignItems="center"
          >
            {amount} {denom}
          </Box>
        ))}
      </Stack>
    </Stack>
  )
}
