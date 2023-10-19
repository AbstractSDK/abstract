import '../styles/globals.css'
import type { AppProps } from 'next/app'
import { ChainProvider, defaultTheme } from '@cosmos-kit/react'
import { ChakraProvider, Container } from '@chakra-ui/react'
import { wallets as keplrWallets } from '@cosmos-kit/keplr'
import { wallets as cosmostationWallets } from '@cosmos-kit/cosmostation'
import { wallets as leapWallets } from '@cosmos-kit/leap'

import { assets, chains } from 'chain-registry'
import { getSigningCosmosClientOptions } from 'interchain'
import { GasPrice } from '@cosmjs/stargate'

import { SignerOptions } from '@cosmos-kit/core'
import { Chain } from '@chain-registry/types'
import { AbstractProvider } from '../contexts/AbstractContext'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Header } from '../components'

const queryClient = new QueryClient()

export default function PaymentApp({ Component, pageProps }: AppProps) {
  const signerOptions: SignerOptions = {
    // @ts-ignore
    signingStargate: (_chain: Chain) => {
      return getSigningCosmosClientOptions()
    },
    signingCosmwasm: (chain: Chain) => {
      // non-ibc fee token
      const feeToken = chain.fees?.fee_tokens?.find((token) => !token.denom.startsWith('ibc/'))
      if (feeToken) {
        return {
          gasPrice: GasPrice.fromString('0.0025' + feeToken.denom),
        }
      }
    },
  }

  return (
    <QueryClientProvider client={queryClient}>
      <ChakraProvider theme={defaultTheme}>
        <ChainProvider
          chains={chains}
          assetLists={assets}
          wallets={[...keplrWallets, ...cosmostationWallets, ...leapWallets]}
          walletConnectOptions={{
            signClient: {
              projectId: '',
              relayUrl: 'wss://relay.walletconnect.org',
              metadata: {
                name: 'Abstract Payment',
                description: '',
                url: '',
                icons: [],
              },
            },
          }}
          wrappedWithChakra={true}
          endpointOptions={{
            endpoints: {
              juno: {
                rpc: ['https://juno-rpc.reece.sh'],
                rest: ['https://juno-api.reece.sh'],
              },
              junotestnet: {
                rpc: ['https://uni-rpc.reece.sh'],
                rest: ['https://uni-api.reece.sh'],
              },
            },
          }}
          signerOptions={signerOptions}
        >
          <AbstractProvider>
            <Header />

            <Container maxW="5xl" py={10}>
              <Component {...pageProps} />
            </Container>
          </AbstractProvider>
        </ChainProvider>
      </ChakraProvider>
    </QueryClientProvider>
  )
}
