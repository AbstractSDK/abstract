import React, { useContext } from 'react'

const createContext = <Value>(name: string, defaultValue?: Value) => {
  const ctx = React.createContext<Value | undefined>(defaultValue)

  const useCtx = () => {
    const context = useContext(ctx)
    if (!context) {
      throw new Error(`use${name} must be used within a ${name}Provider`)
    }

    return context
  }

  return [useCtx, ctx.Provider] as const
}

export default createContext
