import { useRouter } from 'next/router'
import { Box, Heading, Stack, Text } from '@chakra-ui/react'
import { useEffect, useState } from 'react'
import Head from 'next/head'
import {
  AccountBalances,
  AccountEditInfo,
  AccountEditNamespace,
  AccountSetupPayment,
  RequireLogin,
} from '../../components'
import { AccountProvider, useAccount } from '../../contexts'

const InnerAccount = () => {
  const { status, accountInfo, getSigningAccountClient } = useAccount()

  const [loaded, setLoaded] = useState(false)
  const [setup, setSetup] = useState(false)
  useEffect(() => {
    if (status !== 'ready') {
      return
    }

    ;(async () => {
      const accountClient = await getSigningAccountClient()
      const modules = await accountClient.getModules()
      setSetup(modules.some(({ id }) => id === 'abstract:payment'))
      setLoaded(true)
    })()
  }, [status, accountInfo, getSigningAccountClient])

  if (!loaded) {
    return <Text textAlign="center">Loading...</Text>
  } else if (!accountInfo) {
    return <Text textAlign="center">Account not found</Text>
  }

  return (
    <Stack>
      <AccountBalances />
      <AccountEditInfo />
      <AccountEditNamespace />

      {!setup && <AccountSetupPayment />}
    </Stack>
  )
}

export default function Account() {
  const router = useRouter()

  const accountNumber = Number(router.query.id)

  if (isNaN(accountNumber)) {
    return <Text textAlign="center">Invalid account</Text>
  }

  return (
    <AccountProvider accountNumber={accountNumber}>
      <Head>
        <title>Abstract Account #{accountNumber}</title>
        <meta name="description" content={`Abstract Account #${accountNumber}`} />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <Box textAlign="center" mb={12}>
        <Heading
          as="h1"
          fontSize={{ base: '3xl', sm: '4xl', md: '5xl' }}
          fontWeight="extrabold"
          mb={3}
        >
          Abstract Account #{accountNumber}
        </Heading>
      </Box>

      <RequireLogin>
        <InnerAccount />
      </RequireLogin>
    </AccountProvider>
  )
}
