import { Button, Icon } from '@chakra-ui/react'
import { ConnectWalletType } from '../types'
import { IoWallet } from 'react-icons/io5'

export const ConnectWalletButton = ({
  buttonText,
  isLoading,
  isDisabled,
  icon,
  onClick,
}: ConnectWalletType) => (
  <Button
    size="lg"
    isLoading={isLoading}
    isDisabled={isDisabled}
    bgImage="linear-gradient(109.6deg, rgba(157,75,199,1) 11.2%, rgba(119,81,204,1) 83.1%)"
    color="white"
    opacity={1}
    transition="all .5s ease-in-out"
    _hover={{
      bgImage: 'linear-gradient(109.6deg, rgba(157,75,199,1) 11.2%, rgba(119,81,204,1) 83.1%)',
      opacity: 0.75,
    }}
    _active={{
      bgImage: 'linear-gradient(109.6deg, rgba(157,75,199,1) 11.2%, rgba(119,81,204,1) 83.1%)',
      opacity: 0.9,
    }}
    onClick={onClick}
  >
    <Icon as={icon ? icon : IoWallet} mr={2} />
    {buttonText}
  </Button>
)
