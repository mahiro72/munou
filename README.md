# Munou🐱

マルコフ連鎖を用いた人工無能チャットボットの実装です。

## 実装詳細
### 学習フェーズ
- 「吾輩は猫である」のテキストを形態素解析で単語に分解
- 2つ前の状態まで考慮するマルコフ連鎖を実装（連続する3つの単語をグループ化し、最初の2単語から3つ目の単語を予測）
- これにより、自然な文章の生成が可能に

### 応答生成フェーズ:
 - ユーザー入力から形態素解析で名詞を抽出
   - 名詞が見つかった場合：その名詞を含む文章を生成
   - 名詞がない場合：ランダムな文章を生成
 - 文章生成は2つ前までの単語から次の単語を無作為に選択

## 実行例
<img src="https://github.com/user-attachments/assets/d0d06cd4-8f62-44c7-8c48-a99263a22ef4" width=500 />

文脈を考慮したマルコフ連鎖により、ユーザ入力から自然にみえる返答を生成します。(よく読むとみえません。これが無能です。)

## 参考
- https://news.mynavi.jp/techplus/article/rustalgorithm-26/

