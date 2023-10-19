import { Link, Stack } from '@chakra-ui/react'
import { WalletButton } from './WalletButton'
import { useChain } from '@cosmos-kit/react'
import { CHAIN_NAME } from '../../config'
import { useEffect } from 'react'
import { WalletStatus } from '@cosmos-kit/core'
import { useRouter } from 'next/router'
import { ParsedUrlQuery } from 'querystring'
import NextLink from 'next/link'

export const Header = () => (
  <Stack p={4} direction="row" justifyContent="space-between" gap={16}>
    <Stack direction="row" gap={4}>
      <PageLink path="/" label="Home" />
      <PageLink path="/account/new" label="New Account" />
      <PageLink path={/^\/account\/\d+/} label={({ id }) => `Account ${id}`} />
    </Stack>

    <WalletButton />
  </Stack>
)

type PageLinkProps = {
  // If RegExp, will only show if the current path matches the regex.
  path: string | RegExp
  label: string | ((query: ParsedUrlQuery) => string)
}

const PageLink = ({ path, label }: PageLinkProps) => {
  const { asPath, query } = useRouter()

  const isRegex = path instanceof RegExp
  const matching = isRegex ? path.test(asPath) : asPath === path

  if (!matching && isRegex) {
    return null
  }

  return (
    <Link as={NextLink} href={isRegex ? '#' : path} fontWeight={matching ? 'bold' : 'normal'}>
      {typeof label === 'string' ? label : label(query)}
    </Link>
  )
}
