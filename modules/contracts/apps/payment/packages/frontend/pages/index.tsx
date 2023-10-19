import Head from 'next/head'
import { Box, Heading, Text, Stack } from '@chakra-ui/react'
import { Link } from '@chakra-ui/next-js'
import { BsPlusCircleFill } from 'react-icons/bs'
import NextLink from 'next/link'

import { useChain } from '@cosmos-kit/react'
import { WalletStatus } from '@cosmos-kit/core'

import { CHAIN_NAME } from '../config'
import { useAccountsQuery } from '../queries'
import { ReactNode } from 'react'
import { RequireLogin } from '../components'

export default function Home() {
  const { status } = useChain(CHAIN_NAME)

  return (
    <>
      <Head>
        <title>Abstract Payment App</title>
        <meta name="description" content="Payment app for Abstract accounts" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <Box textAlign="center" mb={12}>
        <Heading
          as="h1"
          fontSize={{ base: '3xl', sm: '4xl', md: '5xl' }}
          fontWeight="extrabold"
          mb={3}
        >
          Abstract Accounts
        </Heading>
      </Box>

      <RequireLogin>
        <Connected />
      </RequireLogin>
    </>
  )
}

const Connected = () => {
  const { address } = useChain(CHAIN_NAME)

  const query = useAccountsQuery(address)

  if (!address || query.isLoading) {
    return <Text textAlign="center">Loading...</Text>
  }

  const accounts = query.data || []

  return (
    <>
      <Stack w="full" maxW="xl" mx="auto" gap="1rem" direction="row" justifyContent="center">
        {accounts.map((account) => (
          <AccountLink key={account.accountNumber} id={account.accountNumber}>
            <Text>{account.info.name}</Text>
          </AccountLink>
        ))}

        <AccountLink id="new">
          <BsPlusCircleFill size="2rem" />
        </AccountLink>
      </Stack>
    </>
  )
}

type AccountLinkProps = {
  id: string | number
  children: ReactNode
}

const AccountLink = ({ id, children }: AccountLinkProps) => (
  <Link
    as={NextLink}
    href={`/account/${id}`}
    p="3rem"
    boxShadow="0 0 2px #ccc, 0 0 5px -1px #ccc"
    borderRadius="md"
    display="flex"
    justifyContent="center"
    alignItems="center"
  >
    {children}
  </Link>
)
