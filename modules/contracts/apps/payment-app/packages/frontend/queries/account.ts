import { subgraphRequest } from './subgraph'
import { gql } from '../__generated__/gql'
import { useQuery } from '@tanstack/react-query'
import { CHAIN_NAME } from '../config'

/**
 * Query a single Account's info.
 */
const accountQuerySpec = gql(/* GraphQL */ `
  query Account($chain: ID!, $accountId: ID!) {
    account(chain: $chain, accountId: $accountId) {
      vault {
        baseAsset
        value
      }
      info {
        name
        description
      }
      modules {
        id
      }
    }
  }
`)

export const useAccountQuery = (accountId: number) =>
  useQuery({
    queryKey: ['account', CHAIN_NAME, accountId],
    queryFn: () =>
      subgraphRequest(accountQuerySpec, {
        chain: CHAIN_NAME,
        accountId: accountId.toString(),
      }),
    select: (query) => {
      if (!query?.account) throw new Error('Account not found')
      return query.account
    },
    // TODO:
    // onSuccess: (data) => {
    //   data.forEach((vault) => {
    //     queryClient.setQueryData(vaultQueries.info(chainId, vault.vaultId).queryKey, vault)
    //   })
    // },
  })
