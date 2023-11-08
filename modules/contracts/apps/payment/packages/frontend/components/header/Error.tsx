import { useColorModeValue, Stack, Icon, Text } from '@chakra-ui/react'
import { MouseEventHandler } from 'react'
import { FiAlertTriangle } from 'react-icons/fi'
import { ConnectWalletButton } from './ConnectWalletButton'

export const Error = ({
  buttonText,
  wordOfWarning,
  onClick,
}: {
  buttonText: string
  wordOfWarning?: string
  onClick: MouseEventHandler<HTMLButtonElement>
}) => {
  const bg = useColorModeValue('orange.200', 'orange.300')

  return (
    <Stack>
      <ConnectWalletButton buttonText={buttonText} isDisabled={false} onClick={onClick} />

      {wordOfWarning && (
        <Stack direction="row" borderRadius="md" bg={bg} color="blackAlpha.900" p={4} spacing={1}>
          <Icon as={FiAlertTriangle} mt={1} />
          <Text>
            <Text fontWeight="semibold" as="span">
              Warning:&ensp;
            </Text>
            {wordOfWarning}
          </Text>
        </Stack>
      )}
    </Stack>
  )
}
