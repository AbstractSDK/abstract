import { Button, FormControl, FormLabel, FormErrorMessage, Input, Stack, Textarea } from "@chakra-ui/react"
import { useForm } from "react-hook-form"
import { useAccount } from "../../contexts"

export type AccountEditInfoForm = {
  name: string
  description?: string
}

export const AccountEditInfo = () => {
  const { accountInfo, getSigningAccountClient } = useAccount()

  const {
    handleSubmit,
    register,
    formState: { errors, isSubmitting },
  } = useForm<AccountEditInfoForm>({
    defaultValues: {
      name: accountInfo?.info.name,
      description: accountInfo?.info.description ?? undefined,
    },
  })

  const onSubmit = async ({ name, description }: AccountEditInfoForm) => {
    const accountClient = await getSigningAccountClient()

    // Update info.
    await accountClient.managerClient.updateInfo({
      name,
      description,
    })
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

        <Button mt={4} colorScheme="teal" isLoading={isSubmitting} type="submit">
          Save
        </Button>
      </form>
    </Stack>
  )
}