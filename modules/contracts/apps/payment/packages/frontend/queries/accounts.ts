import { subgraphRequest } from './subgraph'
import { gql } from '../__generated__/gql'
import { useQuery } from '@tanstack/react-query'
import { CHAIN_NAME } from '../config'

/**
 * List Accounts for an owner wallet.
 */
const accountsQuerySpec = gql(/* GraphQL */ `
  query Accounts($chain: ID!, $filter: AccountFilter!) {
    accounts(chain: $chain, chains: [$chain], filter: $filter) {
      id
      accountNumber
      namespace
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

// If owner is not provided, query not enabled.
export const useAccountsQuery = (owner = '') =>
  useQuery({
    queryKey: ['accounts', CHAIN_NAME, owner],
    queryFn: () =>
      subgraphRequest(accountsQuerySpec, {
        chain: CHAIN_NAME,
        filter: {
          owner,
        },
      }),
    select: (query) => {
      if (!query?.accounts) throw new Error('Account not found')
      return query.accounts
    },
    // TODO:
    // onSuccess: (data) => {
    //   data.forEach((vault) => {
    //     queryClient.setQueryData(vaultQueries.info(chainId, vault.vaultId).queryKey, vault)
    //   })
    // },
    enabled: !!owner,
  })
