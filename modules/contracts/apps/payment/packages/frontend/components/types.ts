import { MouseEventHandler, ReactNode } from 'react'
import { IconType } from 'react-icons'

export interface ConnectWalletType {
  buttonText?: string
  isLoading?: boolean
  isDisabled?: boolean
  icon?: IconType
  onClick?: MouseEventHandler<HTMLButtonElement>
}

export interface ConnectedUserCardType {
  walletIcon?: string
  username?: string
  icon?: ReactNode
}

export interface ChainCardProps {
  prettyName: string
  icon?: string
}

export type CopyAddressType = {
  address?: string
  walletIcon?: string
  isLoading?: boolean
  maxDisplayLength?: number
  isRound?: boolean
  size?: string
}
