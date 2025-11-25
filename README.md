# **LangSprint – Multilingual Alphabet & Typing Trainer**

*A modular learning tool for alphabets, frequency vocabulary, and typing fluency across multiple languages.*

LangSprint is a cross-language educational application built to help learners master **alphabets**, **high-frequency vocabulary**, and **typing proficiency** for any language they choose to study.
It currently includes full support for **Georgian**, with the system designed to add more languages as you progress on your learning journey.

The app focuses on **muscle memory**, **visual reinforcement**, and **meaningful input**, blending typing practice with actual language understanding.

---

## **Features**

### **✓ Modular Alphabet System**

* Each language has its own `/langs/<language>/alphabet.json` file
* For each character:

  * Letter representation
  * Name
  * IPA pronunciation
  * Audio playback
  * Finger-position hints (touch-typing guidance)
* Designed to map foreign alphabets onto your **physical QWERTY keyboard**, so you always know where to press.

### **✓ Fully Interactive Keyboard**

* Real-time per-key highlighting
* Shift-left / shift-right tracking (soon)
* Multiple simultaneous keypress support
* Spacebar visualization
* Works with custom key maps per language

Useful for:

* Learning new keyboard layouts
* Typing drills
* Muscle-memory training for foreign alphabets

### **✓ Frequency-Based Typing Test (Top 1000 Words)**

* For each language, the app can load a frequency list:

  * Word in target language
  * English definition
  * Pronunciation
  * Optional example sentences
* Trains both recognition and typing fluency using practical vocabulary.

---

## **Future Language Support**

LangSprint is architected so you can add new languages simply by dropping files into:

```
/langs/<language>/alphabet.json
/langs/<language>/words.json
/langs/<language>/pronunciation/...
```

Planned expansions as you personally learn and study new languages:

* **Georgian** (current)
* **Russian**
* **Spanish**
* **German**
* **Dutch**
* **Swedish**
* **Polish**
* **Hungarian**
* **etc.**
  (Whatever you choose to learn—you can add effortlessly when I make more scripts to digest information.)

---

## **Upcoming Features**

### **→ Grammar Modules**

Per-language grammar lessons such as:

* Case systems
* Verb conjugations
* Core syntax patterns
* Particles & function words
* Inline grammar tips during tests

### **→ Progress & Stats**

* Typing accuracy
* Per-letter difficulty heatmaps
* Words per minute
* Mastery tracking across languages

### **→ Listening & Pronunciation**

* Sentence audio
* Listening-based typing
* Shadowing/pronunciation mimic mode

### **→ Custom Lessons**

* Create your own word lists
* Import vocabulary sets
* Spaced repetition review

---

## **Tech Stack**

* **Rust**
* **Dioxus 0.7**
* **TailwindCSS**
* JSON-based content system
* Real-time input handling with keyboard visualization

---

## **Why This Exists**

LangSprint is designed as a tool to support your long-term multilingual learning journey.

It blends:

* **Typing training**
* **Alphabet acquisition**
* **Vocabulary reinforcement**
* **Interactive bilingual input**

This avoids the trap of learning “words without knowing how to type or pronounce them,” and instead makes you comfortable using the language as a real skill.

---

## **Contributing**

Pull requests, feature requests, and contributions are welcome—especially for additional languages or grammar packs.

---


### Rust && Dioxus Required 
goes without saying, but if you need a guide:

1:
``https://rust-lang.org/learn/get-started/``

2:
```bash
	cargo install dioxus-cli --version 0.7.0
```
## additional (((Windows))) caveats before step 2

install choco
Follow this guide: https://chocolatey.org/install#individual
install nasm
```bash
choco install nasm

then

Win + R "sysdm.cpl" , Advanced -> Enviroment Variables, System variables, Select Path, Edit, Create New -> " C:\Program Files\NASM\ "
```
install cmake
```bash
choco install cmake --installargs 'ADD_CMAKE_TO_PATH=System'
```


### IMPORTANT!
add this line at the top of ``src/main.rs``
```rust
#![windows_subsystem = "windows"]
```

### Please keep in mind for development you must run tailwind in order for the UI to render properly!

### Tailwind
1. Install npm: https://docs.npmjs.com/downloading-and-installing-node-js-and-npm
2. Install the Tailwind CSS CLI: https://tailwindcss.com/docs/installation
```bash
npm install tailwindcss @tailwindcss/cli
```
3. Run the following command in the root of the project to start the Tailwind CSS compiler:

```
	linux:
./tailwind.sh
	(((windows))):
	just the tailwind.sh command manually in a separate terminal
```

### Serving Your App

Run the following command in the root of your project to start developing with the default platform:
 (it will serve desktop by default, will implement web version in the future)
```bash
dx serve
```

To run for a different platform, use the `--platform platform` flag. E.g.
```bash
dx serve --platform desktop
```

if for some reason on linux it doesn't display anything:
```bash
 export WEBKIT_DISABLE_DMABUF_RENDERER=1
 ```

 or you can include it in the command to run this like so:


 ```bash
 WEBKIT_DISABLE_DMABUF_RENDERER=1 dx serve
 ```

### Sources

https://www.hippocrenebooks.com/beginners-georgian-online-audio.html
# Dodona Kiziria - Beginner’s Georgian - Hippocrene Books (2009) (Alphabet Pronunciation Audio)


