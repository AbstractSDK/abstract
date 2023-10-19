import {
  Box,
  Button,
  Container,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Heading,
  Input,
  Stack,
  Textarea,
} from '@chakra-ui/react'
import { findAttribute } from '@cosmjs/stargate/build/logs'
import { useForm } from 'react-hook-form'
import { useChain } from '@cosmos-kit/react'

import { CHAIN_NAME } from '../../config'
import { useRouter } from 'next/router'
import Head from 'next/head'
import { RequireLogin } from '../../components'
import { useAbstract } from '../../contexts'

type NewAccountForm = {
  name: string
  description?: string
}

export default function New() {
  const router = useRouter()
  const { address } = useChain(CHAIN_NAME)
  const { status, getSigningAbstractClient } = useAbstract()
  const {
    handleSubmit,
    register,
    formState: { errors, isSubmitting },
  } = useForm<NewAccountForm>()

  const onSubmit = async (data: NewAccountForm) => {
    const abstractClient = await getSigningAbstractClient()
    if (!address) {
      return
    }

    const { logs } = await abstractClient.factoryClient.createAccount({
      name: data.name,
      description: data.description || undefined,
      governance: {
        Monarchy: {
          monarch: address,
        },
      },
    })
    const accountId = findAttribute(logs, 'wasm-abstract', 'account_id').value

    router.push(`/account/${accountId}`)
  }

  return (
    <>
      <Head>
        <title>New Account</title>
        <meta name="description" content="Create new Abstract account" />
        <link rel="icon" href="/favicon.ico" />
      </Head>

      <Box textAlign="center" mb={12}>
        <Heading
          as="h1"
          fontSize={{ base: '3xl', sm: '4xl', md: '5xl' }}
          fontWeight="extrabold"
          mb={3}
        >
          New Abstract Account
        </Heading>
      </Box>

      <RequireLogin>
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
            <FormControl isInvalid={!!errors.name}>
              <FormLabel htmlFor="name">Account Name</FormLabel>
              <Input
                id="name"
                placeholder="Name"
                {...register('name', {
                  required: 'This is required',
                  minLength: { value: 4, message: 'Minimum length should be 4' },
                })}
              />
              <FormErrorMessage>{errors.name?.message}</FormErrorMessage>
            </FormControl>

            <FormControl isInvalid={!!errors.description}>
              <FormLabel htmlFor="description">Account Description (optional)</FormLabel>
              <Textarea
                id="description"
                placeholder="description"
                {...register('description', {
                  required: 'This is required',
                })}
              />
              <FormErrorMessage>{errors.description?.message}</FormErrorMessage>
            </FormControl>

            <Button
              mt={4}
              colorScheme="teal"
              isLoading={status === 'loading' || isSubmitting}
              disabled={status !== 'ready'}
              type="submit"
            >
              Create
            </Button>
          </form>
        </Stack>
      </RequireLogin>
    </>
  )
}
