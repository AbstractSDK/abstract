/**
* This file was automatically generated by @abstract-money/ts-codegen@0.28.3.
* DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
* and run the @abstract-money/ts-codegen generate command to regenerate this file.
*/

import { CamelCasedProperties } from "type-fest";
import { SigningCosmWasmClient, ExecuteResult } from "@cosmjs/cosmwasm-stargate";
import { AbstractQueryClient, AbstractAccountQueryClient, AbstractAccountClient, AppExecuteMsg, AppExecuteMsgFactory, AbstractClient } from "@abstract-money/abstract.js";
import { StdFee, Coin } from "@cosmjs/amino";
import { InstantiateMsg, ExecuteMsg, QueryMsg, MigrateMsg, ConfigResponse } from "./Template.types";
import { TemplateQueryMsgBuilder, TemplateExecuteMsgBuilder } from "./Template.msg-builder";
export interface ITemplateQueryClient {
  moduleId: string;
  accountQueryClient: AbstractAccountQueryClient;
  _moduleAddress: string | undefined;
  config: () => Promise<ConfigResponse>;
  connectSigningClient: (signingClient: SigningCosmWasmClient, address: string) => TemplateClient;
  getAddress: () => Promise<string>;
}
export class TemplateQueryClient implements ITemplateQueryClient {
  accountQueryClient: AbstractAccountQueryClient;
  moduleId: string;
  _moduleAddress: string | undefined;

  constructor({
    abstractQueryClient,
    accountId,
    managerAddress,
    proxyAddress,
    moduleId
  }: {
    abstractQueryClient: AbstractQueryClient;
    accountId: number;
    managerAddress: string;
    proxyAddress: string;
    moduleId: string;
  }) {
    this.accountQueryClient = new AbstractAccountQueryClient({
      abstract: abstractQueryClient,
      accountId,
      managerAddress,
      proxyAddress
    });
    this.moduleId = moduleId;
    this.config = this.config.bind(this);
  }

  config = async (): Promise<ConfigResponse> => {
    return this._query(TemplateQueryMsgBuilder.config());
  };
  getAddress = async (): Promise<string> => {
    if (!this._moduleAddress) {
      this._moduleAddress = await this.accountQueryClient.getModuleAddress(this.moduleId);
    }

    return this._moduleAddress!;
  };
  connectSigningClient = (signingClient: SigningCosmWasmClient, address: string): TemplateClient => {
    return new TemplateClient({
      accountId: this.accountQueryClient.accountId,
      managerAddress: this.accountQueryClient.managerAddress,
      proxyAddress: this.accountQueryClient.proxyAddress,
      moduleId: this.moduleId,
      abstractClient: this.accountQueryClient.abstract.connectSigningClient(signingClient, address)
    });
  };
  _query = async (queryMsg: QueryMsg): Promise<any> => {
    return this.accountQueryClient.queryModule({
      moduleId: this.moduleId,
      moduleType: "app",
      queryMsg
    });
  };
}
export interface ITemplateClient extends ITemplateQueryClient {
  accountClient: AbstractAccountClient;
  updateConfig: (fee?: number | StdFee | "auto", memo?: string, _funds?: Coin[]) => Promise<ExecuteResult>;
}
export class TemplateClient extends TemplateQueryClient implements ITemplateClient {
  accountClient: AbstractAccountClient;

  constructor({
    abstractClient,
    accountId,
    managerAddress,
    proxyAddress,
    moduleId
  }: {
    abstractClient: AbstractClient;
    accountId: number;
    managerAddress: string;
    proxyAddress: string;
    moduleId: string;
  }) {
    super({
      abstractQueryClient: abstractClient,
      accountId,
      managerAddress,
      proxyAddress,
      moduleId
    });
    this.accountClient = AbstractAccountClient.fromQueryClient(this.accountQueryClient, abstractClient);
    this.updateConfig = this.updateConfig.bind(this);
  }

  updateConfig = async (fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    return this._execute(TemplateExecuteMsgBuilder.updateConfig(), fee, memo, _funds);
  };
  _execute = async (msg: ExecuteMsg, fee: number | StdFee | "auto" = "auto", memo?: string, _funds?: Coin[]): Promise<ExecuteResult> => {
    const moduleMsg: AppExecuteMsg<ExecuteMsg> = AppExecuteMsgFactory.executeApp(msg);
    return await this.accountClient.abstract.client.execute(this.accountClient.sender, await this.getAddress(), moduleMsg, fee, memo, _funds);
  };
}