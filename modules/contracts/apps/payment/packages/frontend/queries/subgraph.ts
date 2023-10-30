import { useQuery } from '@tanstack/react-query'
import request, { RequestDocument, Variables } from 'graphql-request'
import { TypedDocumentNode } from '@graphql-typed-document-node/core'
import { QueryKey } from '@tanstack/query-core'
import { UseQueryOptions } from '@tanstack/react-query/src/types'
import { VariablesAndRequestHeadersArgs } from 'graphql-request/src/types'
import { ABSTRACT_SUBGRAPH_URL } from '../config'

export const subgraphRequest = <TResult = unknown, V extends Variables = Variables>(
  gqlQuery: RequestDocument | TypedDocumentNode<TResult, V>,
  ...variablesAndRequestHeaders: VariablesAndRequestHeadersArgs<V>
) =>
  request(
    ABSTRACT_SUBGRAPH_URL,
    gqlQuery,
    // @ts-ignore
    ...variablesAndRequestHeaders
  )

// /**
//  * Query abstract subgraph with options provided for react query.
//  * @param gqlQuery
//  * @param options
//  * @param variablesAndRequestHeaders
//  */
// export const useSubgraphQuery = <
//   TQueryKey extends QueryKey = QueryKey,
//   TResult = unknown,
//   TData = TResult,
//   V extends Variables = Variables
// >(
//   options: Omit<UseQueryOptions<TResult, Error, TData, TQueryKey>, 'queryFn'>,
//   gqlQuery: RequestDocument | TypedDocumentNode<TResult, V>,
//   ...variablesAndRequestHeaders: VariablesAndRequestHeadersArgs<V>
// ) => {
//   return useQuery({
//     queryFn: async () => subgraphRequest(gqlQuery, ...variablesAndRequestHeaders),
//     ...options,
//   })
// }
