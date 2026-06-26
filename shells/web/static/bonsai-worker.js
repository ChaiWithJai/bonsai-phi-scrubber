import {
  pipeline,
  TextStreamer,
  InterruptableStoppingCriteria,
} from "https://cdn.jsdelivr.net/npm/@huggingface/transformers@4.1.0";

const MODEL_ID = "onnx-community/Bonsai-1.7B-ONNX";
let generatorPromise = null;
let generator = null;
const stoppingCriteria = new InterruptableStoppingCriteria();

function post(status, data = {}) {
  self.postMessage({ status, ...data });
}

async function checkWebGpu() {
  if (!navigator.gpu) throw new Error("navigator.gpu is not exposed");
  const adapter = await navigator.gpu.requestAdapter();
  if (!adapter) throw new Error("WebGPU requestAdapter returned null");
  return adapter;
}

async function load(progress_callback = null) {
  await checkWebGpu();
  if (!generatorPromise) {
    post("loading", { detail: "downloading Bonsai q1 ONNX weights" });
    generatorPromise = pipeline("text-generation", MODEL_ID, {
      device: "webgpu",
      dtype: "q1",
      progress_callback,
    });
  }
  generator = await generatorPromise;
  post("loading", { detail: "warming WebGPU kernels" });
  const inputs = generator.tokenizer("a");
  await generator.model.generate({ ...inputs, max_new_tokens: 1 });
  post("ready", { model: MODEL_ID });
  return generator;
}

function spanMessages(text) {
  return [
    {
      role: "system",
      content:
        "You identify identifiers in synthetic coaching notes. Return JSON only: {\"spans\":[{\"entity\":\"PERSON|MEMBER_ID|DATE|FAMILY_DETAIL|LOCATION|OTHER\",\"text\":\"exact substring\"}]}. Do not rewrite the note.",
    },
    {
      role: "user",
      content: text,
    },
  ];
}

async function generateSpans(text) {
  const gen = generator || (await load());
  let startedAt = 0;
  let numTokens = 0;
  let tps = 0;
  let streamed = "";
  const streamer = new TextStreamer(gen.tokenizer, {
    skip_prompt: true,
    skip_special_tokens: true,
    callback_function: (output) => {
      streamed += output;
      post("update", { output: streamed, tps, numTokens });
    },
    token_callback_function: () => {
      startedAt ||= performance.now();
      numTokens += 1;
      if (numTokens > 1) tps = (numTokens / (performance.now() - startedAt)) * 1000;
    },
  });
  post("start", { detail: "generating browser-local span candidates" });
  const output = await gen(spanMessages(text), {
    max_new_tokens: 384,
    do_sample: false,
    streamer,
    stopping_criteria: stoppingCriteria,
  });
  const content = output?.[0]?.generated_text?.at?.(-1)?.content || streamed;
  post("complete", { output: content, tps, numTokens, model: MODEL_ID });
}

self.addEventListener("message", async (event) => {
  const { type, text } = event.data || {};
  try {
    if (type === "check") {
      await checkWebGpu();
      post("checked", { model: MODEL_ID });
    } else if (type === "load") {
      await load((info) => {
        if (info.status === "progress_total") {
          post("progress", {
            progress: Number(info.progress || 0),
            loaded: Number(info.loaded || 0),
            total: Number(info.total || 0),
          });
        }
      });
    } else if (type === "generate-spans") {
      await generateSpans(String(text || ""));
    } else if (type === "interrupt") {
      stoppingCriteria.interrupt();
      post("interrupted");
    }
  } catch (error) {
    post("error", { detail: error && error.message ? error.message : String(error) });
  }
});
