import createContext from './createContext'
import { ReactNode, useCallback, useEffect, useMemo, useState } from 'react'
import { AbstractAccountClient, AbstractAccountQueryClient } from '@abstract-money/abstract.js'
import { useAccountQuery } from '../queries/account'
import { AccountQuery } from '../__generated__/gql/graphql'
import { useAbstract } from './AbstractContext'

export type AccountContext = {
  // Status of the account client. Initially loading, then ready or error.
  status: 'loading' | 'ready' | 'error'
  // If status is error, the error will be available. Otherwise, it will be
  // undefined.
  error: Error | undefined
  // If status is ready, the account query client will be available. Otherwise,
  // it will be undefined.
  accountQueryClient: AbstractAccountQueryClient | undefined
  // If status is ready, the account info will be available. Otherwise, it will
  // be undefined.
  accountInfo: AccountQuery['account'] | undefined
  // Get the signing account client for the wallet.
  getSigningAccountClient: () => Promise<AbstractAccountClient>
  // In case it fails to load, try to refresh the query client.
  reloadAccountQueryClient: () => Promise<AbstractAccountQueryClient>
}

export type AccountProviderProps = {
  accountNumber: number
  children: ReactNode | ReactNode[]
}

const [useAccount, _AccountProvider] = createContext<AccountContext>('Account')

const AccountProvider = ({ accountNumber, children }: AccountProviderProps) => {
  const { status: abstractStatus, abstractQueryClient, getSigningAbstractClient } = useAbstract()

  const [accountQueryClient, setAccountQueryClient] = useState<
    AbstractAccountQueryClient | undefined
  >()
  const [error, setError] = useState<Error | undefined>()
  const status =
    abstractStatus !== 'ready'
      ? abstractStatus
      : accountQueryClient
      ? 'ready'
      : error
      ? 'error'
      : 'loading'

  const { data: accountInfo } = useAccountQuery(accountNumber)

  const loadQueryClient = useCallback(async () => {
    try {
      if (!abstractQueryClient) {
        throw new Error('Abstract query client not connected')
      }

      const accountQueryClient = await abstractQueryClient.loadAccount(accountNumber)
      setAccountQueryClient(accountQueryClient)
      return accountQueryClient
    } catch (error) {
      setError(error instanceof Error ? error : new Error(`${error}`))
      throw error
    }
  }, [abstractQueryClient, accountNumber])

  // Get account client with wallet signing client.
  const getSigningAccountClient = useCallback(async () => {
    if (!accountQueryClient) {
      throw new Error('Account client not connected')
    }

    const abstractClient = await getSigningAbstractClient()
    return await accountQueryClient.connectAbstractClient(abstractClient)
  }, [accountQueryClient, getSigningAbstractClient])

  const contextValue = useMemo<AccountContext>(
    () => ({
      status,
      error,
      accountQueryClient,
      accountInfo,
      getSigningAccountClient,
      reloadAccountQueryClient: loadQueryClient,
    }),
    [status, error, accountQueryClient, accountInfo, getSigningAccountClient, loadQueryClient]
  )

  // Load query client once abstract client is ready.
  useEffect(() => {
    if (abstractStatus === 'ready') {
      loadQueryClient()
    }
  }, [abstractStatus, loadQueryClient])

  return <_AccountProvider value={contextValue}>{children}</_AccountProvider>
}

export { AccountProvider, useAccount }
