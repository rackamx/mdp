# CommonMark Stress Document

This file is intended to exercise a broad set of CommonMark parsing and rendering rules.
If your renderer handles this file correctly, it likely covers most core spec behavior.

---

## 1. ATX Headings

# H1
## H2
### H3
#### H4
##### H5
###### H6

####### Not a heading (7 hashes should be plain text)

## ATX closing hashes ### ###

---

## 2. Setext Headings

Setext H1
=========

Setext H2
---------

Paragraph
---------
and continuation text.

---

## 3. Paragraphs, Soft Breaks, Hard Breaks

This line
wraps with a soft break.

This line ends with two spaces.  
This should be a hard break.

This line ends with a backslash.\
This should also be a hard break.

---

## 4. Thematic Breaks

***

---

___

Not a thematic break: - - (too short)

---

## 5. Block Quotes

> Single level quote.
>
> Multiple paragraphs inside quote.
>
> > Nested quote level 2.
> >
> > - list in nested quote
> > - second item
>
> Back to level 1.

> Quote with heading
> ## Quoted heading
>
> And fenced code:
> ```rust
> let x = 1;
> ```

---

## 6. Lists

### 6.1 Unordered Lists

- dash item 1
- dash item 2
  - nested level 2
    - nested level 3
- dash item 3

* star item 1
* star item 2

+ plus item 1
+ plus item 2

### 6.2 Ordered Lists

1. one
2. two
3. three

3. starts at three
4. next item

1. nested ordered
   1. child ordered
   2. second child
2. back to parent

### 6.3 Loose vs Tight Lists

- tight a
- tight b

- loose a

- loose b

### 6.4 Mixed List Content

- item with paragraph continuation
  still same list item paragraph

  new paragraph in same item

  > block quote in list item

  ```text
  code block in list item
  ```

- second item

---

## 7. Code Blocks

### 7.1 Indented Code Block

    indented code line 1
    indented code line 2

### 7.2 Fenced Code Blocks

````
fence with four backticks
and ``` inside content
````

```rust
fn main() {
    println!("hello");
}
```

``` unknown-lang
language info string with space
```

~~~
tilde fence
~~~

---

## 8. Inline Code

Use `code` inline.

Use ``code with `backtick` inside`` inline.

Use `` `surrounded by spaces` `` style delimiters.

---

## 9. Emphasis and Strong Emphasis

*emphasis with asterisks*

_emphasis with underscores_

**strong with asterisks**

__strong with underscores__

***strong+emphasis***

___strong+emphasis___

**nested *inner emphasis* text**

*nested **inner strong** text*

un*frigging*believable (intraword)

This_is_not_always_emphasis_in_words (underscore edge case)

---

## 10. Links

Inline link: [CommonMark](https://commonmark.org).

Inline link with title: [Example](https://example.com "Example Title").

Autolink URL: <https://example.com/path?q=1&x=2>.

Autolink email: <user.name+tag@example.com>.

Reference link: [ref-link][id1].

Collapsed ref: [id2][].

Shortcut ref: [id3].

Literal brackets in text: \[not a link\].

---

## 11. Images

Inline image: ![alt text](https://example.com/image.png "Image Title")

Reference image: ![ref image][img1]

Image-like text escaped: \![not image](x)

---

## 12. Escapes and Entities

Escaped punctuation: \* \_ \` \\ \[ \] \( \) \# \+ \- \. \!

Escaped thematic break candidate: \---

Entity named: &copy; &amp; &lt; &gt;

Entity decimal: &#169;

Entity hex: &#xA9;

Unknown entity should remain text: &notanentity;

---

## 13. Raw HTML (CommonMark HTML blocks and inline HTML)

Inline HTML: <span data-x="1">inline span</span> with surrounding text.

<div class="block-html">
This is HTML block content.
<em>HTML emphasis</em> should stay as raw HTML in CommonMark parsing.
</div>

<!-- HTML comment block -->

<?processing instruction?>

<![CDATA[
some cdata-like content
]]>

---

## 14. Backslash Escapes at Line Ends

line one\
line two\
line three

---

## 15. Tabs and Indentation

	This line starts with a tab.

		Two tabs.

1.	list marker followed by tab
2.	second tabbed item

---

## 16. Delimiter and Precedence Edge Cases

*a **b* c**

**a *b** c*

`*not emphasis inside code*`

[link with `code` inside](https://example.com)

<https://example.com/**not-strong**>

Paragraph with <em>inline html</em> and *markdown emphasis* mixed.

---

## 17. Blank Line Sensitivity

Text before.


Text after two blank lines.

- list item one

- list item two separated by blank line

> quote one
>
> quote two

---

## 18. Reference Definitions

[id1]: https://example.com/ref "Ref One"
[id2]: https://example.com/collapsed
[id3]: https://example.com/shortcut
[img1]: https://example.com/ref-image.png "Ref Image"

---

## 19. Extensions (Not Core CommonMark, Optional)

These are intentionally included to test non-core behavior if supported.

| Table | Col B | Col C |
| :---- | ----: | :---: |
| left  | right | center |

- [ ] task unchecked
- [x] task checked

~~strikethrough extension~~

Footnote style (extension): reference[^1]

[^1]: Footnote text (extension)

