import { chains, assets } from 'chain-registry'
import { Asset } from '@chain-registry/types'

export const CHAIN_NAME = process.env.NEXT_PUBLIC_CHAIN_NAME || 'junotestnet'

export const CHAIN_ASSETS: Asset[] =
  assets.find((chain) => chain.chain_name === CHAIN_NAME)?.assets || []

export const CHAIN_FEE_DENOM = chains
  .find((chain) => chain.chain_name === CHAIN_NAME)
  ?.fees?.fee_tokens?.find((asset) => !asset.denom.startsWith('ibc/'))?.denom
if (!CHAIN_FEE_DENOM) {
  throw new Error('Fee denom not found')
}

export const CHAIN_FEE_ASSET = CHAIN_ASSETS.find((asset) => asset.base === CHAIN_FEE_DENOM)
if (!CHAIN_FEE_ASSET) {
  throw new Error('Fee asset not found')
}

export const FACTORY_CONTRACT_ADDRESS = process.env.NEXT_PUBLIC_FACTORY_CONTRACT_ADDRESS!

export const ABSTRACT_SUBGRAPH_URL = 'https://abstract-subgraph-0-17.fly.dev/'
