/* tslint:disable */
/* eslint-disable */
export function start(): void;
export function wasm_logger_init(): void;
export function initialize_game(canvas_id: string): GameInstance;
/**
 * ゲームインスタンス
 *
 * ゲームの状態とシステムを管理する主要な構造体です。
 * ゲームループの制御、リソースの管理、エンティティの管理を行います。
 */
export class Game {
  free(): void;
  /**
   * 新しいゲームインスタンスを作成します。
   *
   * # 引数
   *
   * * `canvas_id` - ゲームの描画先キャンバスのID
   *
   * # 戻り値
   *
   * 初期化されたGameインスタンス、または初期化エラー
   */
  constructor(canvas_id: string);
  /**
   * ゲームのメインループを1フレーム進めます。
   *
   * # 引数
   *
   * * `delta_time` - 前フレームからの経過時間（秒）
   */
  update(delta_time: number): void;
  /**
   * ゲームを描画します。
   */
  render(): void;
  /**
   * キー入力を処理します。
   *
   * # 引数
   *
   * * `key_code` - キーコード
   * * `pressed` - キーが押されたかどうか
   */
  handle_key_input(key_code: number, pressed: boolean): void;
  /**
   * マウス入力を処理します。
   *
   * # 引数
   *
   * * `x` - マウスのX座標
   * * `y` - マウスのY座標
   * * `button` - マウスボタン
   * * `pressed` - ボタンが押されたかどうか
   */
  handle_mouse_input(x: number, y: number, button: number, pressed: boolean): void;
}
export class GameInstance {
  private constructor();
  free(): void;
  static new(canvas_id: string): GameInstance;
  connect_to_server(server_url: string): void;
  disconnect_from_server(): void;
  get_connection_state(): string;
  update(): number;
  render(): void;
  /**
   * キーイベントを処理
   */
  handle_key_event(key_code: number): void;
  handle_mouse_event(event_type: string, x: number, y: number, button?: number | null): void;
  dispose(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_game_free: (a: number, b: number) => void;
  readonly game_new: (a: number, b: number) => [number, number, number];
  readonly game_update: (a: number, b: number) => [number, number];
  readonly game_render: (a: number) => [number, number];
  readonly game_handle_key_input: (a: number, b: number, c: number) => [number, number];
  readonly game_handle_mouse_input: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly start: () => void;
  readonly initialize_game: (a: number, b: number) => [number, number, number];
  readonly __wbg_gameinstance_free: (a: number, b: number) => void;
  readonly gameinstance_new: (a: number, b: number) => [number, number, number];
  readonly gameinstance_connect_to_server: (a: number, b: number, c: number) => [number, number];
  readonly gameinstance_disconnect_from_server: (a: number) => [number, number];
  readonly gameinstance_get_connection_state: (a: number) => [number, number];
  readonly gameinstance_update: (a: number) => number;
  readonly gameinstance_render: (a: number) => void;
  readonly gameinstance_handle_key_event: (a: number, b: number) => [number, number];
  readonly gameinstance_handle_mouse_event: (a: number, b: number, c: number, d: number, e: number, f: number) => void;
  readonly gameinstance_dispose: (a: number) => void;
  readonly wasm_logger_init: () => void;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_export_2: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_6: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly closure2_externref_shim: (a: number, b: number, c: any) => void;
  readonly closure94_externref_shim: (a: number, b: number, c: any) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
