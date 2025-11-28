"""
Prompt templates for different summarization tasks.

These prompts are designed for local LLMs to generate
concise, accurate documentation for code.
"""

# System prompts - define the model's role and behavior
SYSTEM_PROMPT_TECHNICAL = """You are a technical documentation expert. Your job is to analyze code and write clear, concise summaries that help developers understand the purpose and functionality of code. Focus on WHAT the code does and WHY, not HOW (the code itself shows how). Be accurate and avoid speculation."""

SYSTEM_PROMPT_ARCHITECTURE = """You are a software architect analyzing codebases. Your job is to identify high-level patterns, relationships between components, and overall system structure. Focus on the big picture and how pieces fit together."""


# Chunk-level summarization
CHUNK_SUMMARY_PROMPT = """Analyze this code and write a 2-3 sentence summary.

Focus on:
- What is the main purpose of this code?
- What functionality does it provide?
- What are the key classes/functions?

Be concise and technical. Do not explain implementation details.

Code:
"""

# For code chunks with context about their file
CHUNK_SUMMARY_WITH_CONTEXT = """Analyze this code from {filepath} and write a 2-3 sentence summary.

Focus on:
- What is the main purpose of this code?
- What functionality does it provide?
- How does it fit into the larger codebase?

Be concise and technical.

Code:
"""


# Cluster-level summarization (groups of related chunks)
CLUSTER_SUMMARY_PROMPT = """You are analyzing a group of semantically related code chunks. Write a summary that explains:

1. What common theme or functionality these chunks share
2. What role this group plays in the overall system
3. Key patterns or important functionality

The chunks below are related because they were grouped by semantic similarity.

Write 1 paragraph (3-5 sentences) summarizing this cluster.

Related code chunks:
"""


# Project-level overview
PROJECT_OVERVIEW_PROMPT = """Based on the cluster summaries below, write a high-level architecture overview of this codebase.

Your overview should include:
1. Primary purpose of the project
2. Major components and their responsibilities
3. How components interact with each other
4. Key technologies or patterns used

Write 2-3 paragraphs. Be clear and concise.

Cluster summaries:
"""


# Function-specific prompt
FUNCTION_SUMMARY_PROMPT = """Summarize this function in 1-2 sentences.

Explain:
- What the function does (its purpose)
- What it returns (if applicable)
- Any important side effects

Function code:
"""


# Class-specific prompt
CLASS_SUMMARY_PROMPT = """Summarize this class in 2-3 sentences.

Explain:
- What this class represents or manages
- Its main responsibilities
- How it's intended to be used

Class code:
"""


# Module/file-level prompt
MODULE_SUMMARY_PROMPT = """Summarize this module/file in 2-3 sentences.

Explain:
- The main purpose of this file
- What functionality it exports or provides
- How it fits into the larger system

Module code:
"""


def build_chunk_prompt(code: str, filepath: str = "") -> str:
    """
    Build a prompt for chunk-level summarization.

    Args:
        code: The code to summarize
        filepath: Optional filepath for context

    Returns:
        Complete prompt string
    """
    if filepath:
        return CHUNK_SUMMARY_WITH_CONTEXT.format(filepath=filepath) + "\n" + code
    return CHUNK_SUMMARY_PROMPT + "\n" + code


def build_cluster_prompt(chunk_summaries: list[str]) -> str:
    """
    Build a prompt for cluster-level summarization.

    Args:
        chunk_summaries: List of individual chunk summaries

    Returns:
        Complete prompt string
    """
    chunks_text = "\n\n---\n\n".join(
        f"Chunk {i+1}:\n{summary}"
        for i, summary in enumerate(chunk_summaries)
    )
    return CLUSTER_SUMMARY_PROMPT + "\n\n" + chunks_text


def build_project_prompt(cluster_summaries: list[str]) -> str:
    """
    Build a prompt for project-level overview.

    Args:
        cluster_summaries: List of cluster summaries

    Returns:
        Complete prompt string
    """
    clusters_text = "\n\n---\n\n".join(
        f"Cluster {i+1}:\n{summary}"
        for i, summary in enumerate(cluster_summaries)
    )
    return PROJECT_OVERVIEW_PROMPT + "\n\n" + clusters_text
