import {
  Button,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Input,
  Stack,
} from '@chakra-ui/react'
import { useForm } from 'react-hook-form'
import { CHAIN_NAME } from '../../config'
import { useChain } from '@cosmos-kit/react'
import { useEffect, useState } from 'react'
import { useAbstract, useAccount } from '../../contexts'


type AccountEditNamespaceForm = {
  namespace?: string
}

export const AccountEditNamespace = () => {
  const { abstractQueryClient } = useAbstract()
  const { accountQueryClient } = useAccount()
  const { address, getSigningCosmWasmClient } = useChain(CHAIN_NAME)

  const {
    setValue,
    handleSubmit,
    register,
    formState: { errors, isSubmitting },
  } = useForm<AccountEditNamespaceForm>()

  const [loadedNamespace, setLoadedNamespace] = useState(false)
  useEffect(() => {
    if (loadedNamespace) {
      return
    }

    accountQueryClient?.getNamespace().then((namespace) => {
      if (namespace) {
        setValue('namespace', namespace)
      }
      setLoadedNamespace(true)
    })
  }, [accountQueryClient, loadedNamespace, setValue])

  const onSubmit = async ({ namespace }: AccountEditNamespaceForm) => {
    const signingCosmWasmClient = await getSigningCosmWasmClient()
    if (!address || !abstractQueryClient || !accountQueryClient) {
      return
    }

    const currentNamespace = await accountQueryClient.getNamespace()
    if (currentNamespace === namespace) {
      return
    }

    try {
      await signingCosmWasmClient.executeMultiple(
        address,
        [
          // Remove existing namespace.
          ...(currentNamespace
            ? [
                {
                  contractAddress: abstractQueryClient.registryAddress,
                  funds: [],
                  msg: {
                    remove_namespaces: {
                      namespaces: [currentNamespace],
                    },
                  },
                },
              ]
            : []),
          // Claim new namespace.
          {
            contractAddress: abstractQueryClient.registryAddress,
            funds: [],
            msg: {
              claim_namespace: {
                account_id: accountQueryClient.accountId,
                namespace,
              },
            },
          },
        ],
        'auto'
      )
    } catch (err) {
      console.error(err)
    } finally {
      // Reload namespace.
      accountQueryClient.getNamespace().then((namespace) => {
        if (namespace) {
          setValue('namespace', namespace)
        }
      })
    }
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
        <FormControl isInvalid={!!errors.namespace}>
          <FormLabel htmlFor="namespace">Namespace</FormLabel>
          <Input
            id="namespace"
            placeholder="Namespace"
            {...register('namespace', {
              required: 'Namespace is required.',
              minLength: {
                value: 3,
                message: 'Namespace must be at least 3 characters.',
              },
              validate: {
                nonNumeric: (value) =>
                  (value && /^[^0-9].*$/.test(value)) || 'Namespace must start with a letter.',
              },
            })}
          />
          <FormErrorMessage>{errors.namespace?.message}</FormErrorMessage>
        </FormControl>

        <Button mt={4} colorScheme="teal" isLoading={isSubmitting} type="submit">
          Save
        </Button>
      </form>
    </Stack>
  )
}