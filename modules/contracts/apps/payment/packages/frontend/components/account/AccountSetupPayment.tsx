import { InstantiateMsg } from 'app-sdk/src/contracts/Template.types'
import {
  Button,
  FormControl,
  FormErrorMessage,
  FormLabel,
  Select,
  Stack,
} from '@chakra-ui/react'
import { useForm } from 'react-hook-form'
import { CHAIN_ASSETS, CHAIN_NAME } from '../../config'
import { useChain } from '@cosmos-kit/react'
import { toBinary } from '@cosmjs/cosmwasm-stargate'
import { useAccount } from '../../contexts'

export type AccountSetupPaymentForm = {
  denom: string
}

export const AccountSetupPayment = () => {
  const { getSigningAccountClient } = useAccount()
  const { address } = useChain(CHAIN_NAME)

  const {
    handleSubmit,
    register,
    formState: { errors, isSubmitting },
  } = useForm<AccountSetupPaymentForm>({
    defaultValues: {
      denom: '',
    },
  })

  const onSubmit = async ({ denom }: AccountSetupPaymentForm) => {
    const accountClient = await getSigningAccountClient()
    if (!address) {
      return
    }

    try {
      // TODO: Fill in info.
      const paymentInit: InstantiateMsg = {
        desired_asset: null,
        exchanges: [],
      }

      // Install DEX and Payment modules. Payment depends on DEX, so install DEX
      // first.
      const currentModules = await accountClient.getModules()
      if (!currentModules.some(({ id }) => id === 'abstract:dex')) {
        await accountClient.managerClient.installModule({
          module: {
            namespace: 'abstract',
            name: 'dex',
            version: 'latest',
          },
        })
      }
      if (!currentModules.some(({ id }) => id === 'abstract:payment')) {
        await accountClient.managerClient.installModule({
          module: {
            namespace: 'abstract',
            name: 'payment',
            version: 'latest',
          },
          initMsg: toBinary({
            base: {
              ans_host_address: accountClient.abstract.ansHostAddress,
            },
            module: paymentInit,
          }),
        })
      }
    } catch (e) {
      console.error(e)
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
          Set up
        </Button>
      </form>
    </Stack>
  )
}
