import { useRouter } from 'next/router'

import { useEffect, useState } from 'react'
import {
  Box,
  Button,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Heading,
  Input,
  Select,
  Stack,
  Text,
} from '@chakra-ui/react'
import { contracts } from 'app-sdk'
import { useForm } from 'react-hook-form'
import { coins } from '@cosmjs/proto-signing'
import { CHAIN_ASSETS } from '../../config'
import { TemplateClient } from 'app-sdk/types/Template.client'
import Head from 'next/head'
import { RequireLogin } from '../../components'
import { AccountProvider, useAbstract, useAccount } from '../../contexts'

type PaymentForm = {
  amount: number
  denom: string
}

const InnerPayAccount = () => {
  const { status, accountInfo, getSigningAccountClient } = useAccount()

  const [loaded, setLoaded] = useState(false)
  const [paymentClient, setPaymentClient] = useState<TemplateClient>()
  useEffect(() => {
    if (status !== 'ready') {
      return
    }

    ;(async () => {
      const accountClient = await getSigningAccountClient()
      const modules = await accountClient.getModules()

      const paymentModule = modules.find(({ id }) => id === 'abstract:payment')
      if (paymentModule) {
        setPaymentClient(
          new contracts.Template.TemplateClient({
            abstractClient: accountClient.abstract,
            accountId: accountClient.accountId,
            managerAddress: accountClient.managerAddress,
            proxyAddress: accountClient.proxyAddress,
            moduleId: paymentModule.id,
          })
        )
      }

      setLoaded(true)
    })()
  }, [accountInfo, status, getSigningAccountClient])

  const {
    handleSubmit,
    register,
    formState: { errors, isSubmitting },
  } = useForm<PaymentForm>({
    defaultValues: {
      amount: 1,
      denom: '',
    },
  })

  const onSubmit = async ({ amount, denom }: PaymentForm) => {
    if (!paymentClient) {
      return
    }

    try {
      const { transactionHash } = await paymentClient.tip('auto', undefined, coins(amount, denom))
      console.log(transactionHash)
      alert(transactionHash)
    } catch (e) {
      console.error(e)
    }
  }

  if (!loaded) {
    return <Text textAlign="center">Loading...</Text>
  } else if (!accountInfo) {
    return <Text textAlign="center">Account not found</Text>
  } else if (!paymentClient) {
    return <Text textAlign="center">Payment module not setup</Text>
  }

  return (
    <Stack
      w="full"
      maxW="xl"
      mx="auto"
      gap="1rem"
      boxShadow="0 0 2px #ccc, 0 0 5px -1px #ccc"
      borderRadius="md"
      p={5}
    >
      <form onSubmit={handleSubmit(onSubmit)}>
        <FormControl isInvalid={!!errors.amount}>
          <FormLabel htmlFor="amount">Amount</FormLabel>
          <Input
            id="amount"
            {...register('amount', {
              required: 'This is required',
              valueAsNumber: true,
            })}
          />
          <FormErrorMessage>{errors.amount?.message}</FormErrorMessage>
        </FormControl>

        <FormControl isInvalid={!!errors.denom}>
          <FormLabel htmlFor="denom">Denom</FormLabel>
          <Select
            id="denom"
            {...register('denom', {
              required: 'This is required',
            })}
          >
            {CHAIN_ASSETS.map((asset) => (
              <option key={asset.base} value={asset.base}>
                {asset.name} ({asset.symbol})
              </option>
            ))}
          </Select>
          <FormErrorMessage>{errors.denom?.message}</FormErrorMessage>
        </FormControl>

        <Button mt={4} colorScheme="teal" isLoading={isSubmitting} type="submit">
          Pay
        </Button>
      </form>
    </Stack>
  )
}

export default function PayAccount() {
  const router = useRouter()
  const { status, abstractQueryClient } = useAbstract()

  const accountId = router.query.account

  const [accountNumber, setAccountNumber] = useState<number>()
  useEffect(() => {
    if (!router.isReady || status !== 'ready' || !abstractQueryClient) {
      return
    }

    if (typeof accountId !== 'string') {
      setAccountNumber(-1)
      return
    }

    // If accountId is a number, assume it's a valid account number.
    if (!isNaN(Number(accountId))) {
      setAccountNumber(Number(accountId))
      return
    }

    // Otherwise, try to look up the account number by namespace.
    ; (async () => {
      try {
        const account = await abstractQueryClient.registryQueryClient.namespace({
          namespace: accountId,
        })
        if (account) {
          setAccountNumber(account.account_id)
        } else {
          setAccountNumber(-1)
        }
      } catch (e) {
        console.error(e)
        setAccountNumber(-1)
      }
    })()
  }, [abstractQueryClient, accountId, router.isReady, status])

  if (accountNumber === undefined) {
    return <Text textAlign="center">Loading...</Text>
  } else if (accountNumber === -1) {
    return <Text textAlign="center">Invalid account</Text>
  }

  return (
    <AccountProvider accountNumber={accountNumber}>
      <Head>
        <title>Pay Account #{accountNumber}</title>
        <meta name="description" content={`Pay Account #${accountNumber}`} />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <Box textAlign="center" mb={12}>
        <Heading
          as="h1"
          fontSize={{ base: '3xl', sm: '4xl', md: '5xl' }}
          fontWeight="extrabold"
          mb={3}
        >
          Pay Account #{accountNumber}
        </Heading>
      </Box>

      <RequireLogin>
        <InnerPayAccount />
      </RequireLogin>
    </AccountProvider>
  )
}
