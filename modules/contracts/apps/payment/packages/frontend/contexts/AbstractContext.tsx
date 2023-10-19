import createContext from './createContext'
import { ReactNode, useCallback, useEffect, useMemo, useState } from 'react'
import { AbstractQueryClient, AbstractClient } from '@abstract-money/abstract.js'
import { useChain } from '@cosmos-kit/react'
import { ABSTRACT_SUBGRAPH_URL, CHAIN_NAME } from '../config'

export type AbstractContext = {
  // Status of the abstract client. Initially loading, then ready or error.
  status: 'loading' | 'ready' | 'error'
  // If status is error, the error will be available. Otherwise, it will be
  // undefined.
  error: Error | undefined
  // If status is ready, the abstract query client will be available. Otherwise,
  // it will be undefined.
  abstractQueryClient: AbstractQueryClient | undefined
  // Get the signing abstract client for the wallet.
  getSigningAbstractClient: () => Promise<AbstractClient>
  // In case it fails to load, try to refresh the query client.
  reloadAbstractQueryClient: () => Promise<AbstractQueryClient>
}

export type AbstractProviderProps = {
  children: ReactNode | ReactNode[]
}

const [useAbstract, _AbstractProvider] = createContext<AbstractContext>('Abstract')

const AbstractProvider = ({ children }: AbstractProviderProps) => {
  const [abstractQueryClient, setAbstractQueryClient] = useState<AbstractQueryClient | undefined>()
  const [error, setError] = useState<Error | undefined>()
  const status = abstractQueryClient ? 'ready' : error ? 'error' : 'loading'

  const {
    chain: { chain_id },
    getSigningCosmWasmClient,
    isWalletConnected,
    address,
  } = useChain(CHAIN_NAME)

  const loadQueryClient = useCallback(async () => {
    try {
      const abstractQueryClient = await AbstractQueryClient.connectToChain(
        chain_id,
        ABSTRACT_SUBGRAPH_URL
      )
      setAbstractQueryClient(abstractQueryClient)
      return abstractQueryClient
    } catch (error) {
      setError(error instanceof Error ? error : new Error(`${error}`))
      throw error
    }
  }, [chain_id])

  // Get abstract client with wallet signing client.
  const getSigningAbstractClient = useCallback(async () => {
    if (!abstractQueryClient) {
      throw new Error('Abstract client not connected')
    }
    if (!isWalletConnected || !address) {
      throw new Error('Wallet not connected')
    }

    const client = await getSigningCosmWasmClient()
    return await abstractQueryClient.connectSigningClient(client, address)
  }, [abstractQueryClient, address, getSigningCosmWasmClient, isWalletConnected])

  const contextValue = useMemo<AbstractContext>(
    () => ({
      status,
      error,
      abstractQueryClient,
      getSigningAbstractClient,
      reloadAbstractQueryClient: loadQueryClient,
    }),
    [status, error, abstractQueryClient, getSigningAbstractClient, loadQueryClient]
  )

  // Load query client on mount.
  useEffect(() => {
    loadQueryClient()
  }, [loadQueryClient])

  return <_AbstractProvider value={contextValue}>{children}</_AbstractProvider>
}

export { AbstractProvider, useAbstract }
