---
marp: true
theme: gaia
size: 16:9
paginate: true
headingDivider: 2
header: 2022-07-23 ｽﾀｯｸﾁｬﾝ誕生会 LT ﾜｲｵﾀｰﾐﾅﾙﾁｬﾝ作ってみた
footer: (C) 2022 Kenta Ida
---

# ｽﾀｯｸﾁｬﾝ誕生会 LT <br/> ﾜｲｵﾀｰﾐﾅﾙﾁｬﾝ作ってみた

<!--
_class: lead
_paginate: false
_header: ""
-->

## 自己紹介

![bg right:30% fit](figure/introduction.drawio.svg)

* 井田　健太 (いだ　けんた)
* twitter: @ciniml
* M5Stack沼: 第4層くらい
* 組込みソフト、FPGA屋
* ATOM DisplayのFPGA設計しました
  * Interface 2022年7月号に解説記事書きました
  * https://interface.cqpub.co.jp/magazine/202207/

## Wio Terminalについて

* ワイオターミナルと読む (Wio = Wireless I/O)
* Seeed Studioが出している液晶・無線つきマイコンモジュール
* はやい話がM5Stack Coreの対抗製品

![bg right:30% fit](figure/wio_terminal.drawio.svg)

## Wio Terminalの利点・欠点

![bg right:30% fit](figure/wio_terminal.drawio.svg)

* 良い点
  * バッテリ非内蔵
  * 物理電源スイッチ
  * 拡張ピン数多い
  * デバッガ使える
* 悪い点
  * 🦀(RTL8720)と🦀のファームウェアのせいで無線が不安定
  * 液晶がIPSじゃないので視野角が狭い

## Wio Terminal-chanについて

![bg right:30% fit](figure/wio_terminal_chan.drawio.svg)

* Seeed Studio公式キャラクター
* Wio Terminalのページに.aiファイルあり
* Wio Terminalが出てしばらくして追加された
* ステッカーなどに登場
* なんかカワイイ

## Wio Terminal-chan型ネックストラップ

![bg right:30% fit](figure/wio_terminal_chan_strap.drawio.svg)

* Maker Faire Tokyo 2020 1日目にSeeedブースにM5Stackをぶら下げていったら ~~怒られが発生~~ 
* 1日目帰宅後、Wio Terminal用ネックストラップを適当に設計
* どうせならと Wio Terminal-chan型にした
* 乾電池駆動

## 2021年7月某日

* ｽｰﾊﾟｰｶﾜｲｲﾛﾎﾞｯﾄ ｽﾀｯｸﾁｬﾝ 登場!
* ｽｰﾊﾟｰｶﾜｲｲ ので、さっそく作ろうと思ったが、そのままだと面白くないな？とか言い出す
* M5StackじゃなくてWio Terminalにしよう
  * ファームはRustで書けばいいんじゃない？

## 余談: 組込みRustと私

![bg right:20% fit](figure/embedded_rust.drawio.svg)

* 技術書典7でM5Stack Rust本を頒布
* 共著でWio Terminalを使った組込みRust本を書いた
  * 最後の方のアプリケーションのところだけ
* 組込みRustたのしいよ

## 2021年8月(1/3)

![bg right:30% fit](figure/design_chassis.drawio.svg)

* 適当に筐体を設計して組み立てる
    * 試行錯誤したのでいくつかゴミが発生
## 2021年8月(2/3)

![bg right:30% fit](figure/wio_terminal_chan_board.drawio.svg)

* ﾜｲｵﾀｰﾐﾅﾙﾁｬﾝ用基板を設計する
  * Wio Terminalは電源内蔵してないので外部電源が必要
  * Wio Terminal-chanで使った乾電池駆動基板を改造
  * サーボ信号出力できるようにコネクタを配置
  * 
## 2021年8月(3/3)

* 発注していた基板が到着するが、実装するのが面倒で放置する

## 2022年6月

* ししかわさんがMFT 2022当選
* ｽﾀｯｸﾁｬﾝ展示とのことなので、MFTまでにﾜｲｵﾀｰﾐﾅﾙﾁｬﾝを完成させようと思いつつまだ時間あるやろとか言いながら放置

## 2022年7月6日

* ｽﾀｯｸﾁｬﾝ誕生会イベント公開
* オフライン/オンライン開催
* LT枠がある
* ﾜｲｵﾀｰﾐﾅﾙﾁｬﾝ完成させろというお達しに違いない

## 2022年7月18日

![bg right:40% fit](figure/build_hardware.drawio.svg)

* ようやくﾜｲｵﾀｰﾐﾅﾙﾁｬﾝ基板を組み立てて、1年間放置していた筐体に組み込む
* ファームウェアはまだ

## 2022年7月21日

![bg right:50% fit](figure/writing_firmware.drawio.svg)

* ようやくファームウェアを書き始める
  * もちろんRustですよ
* とりあえずPWM出力してサーボモータをランダムに動かす部分は完成

## 2022年7月23日

![bg right:50% fit](figure/writing_firmware_face.drawio.svg)

* MFTで作ったコードを参考に、ﾜｲｵﾀｰﾐﾅﾙﾁｬﾝの顔の描画処理を作る
* まあなんとかなったかな

## デモ？

* 実物あるのでみてもらえれば。

## 作ってみた感想

* ｽﾀｯｸﾁｬﾝの設計はよくできている
  * ﾜｲｵﾀｰﾐﾅﾙﾁｬﾝはワンオフ品としてつくっているので、数を作るためには最適化の余地がある
* ｽｰﾊﾟｰｶﾜｲｲ挙動にするにはソフトが重要
  * 単に動かすだけだと結構微妙
  * でもまあなんかカワイイからいいか
* Rustはたのしい
* Wio Terminalもいい子なのでさわってみてください。