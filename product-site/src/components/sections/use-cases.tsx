"use client";

import { useRef } from "react";
import { motion, useScroll, useTransform, useInView } from "framer-motion";
import { Badge } from "../ui/badge";
import { Card } from "../ui/card";

const useCases = [
  {
    category: "Home Care",
    color: "coral" as const,
    gradient: "from-[var(--pop-coral)] to-[var(--pop-orange)]",
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
      </svg>
    ),
    tasks: [
      "Change air filters every 30 days",
      "Deep clean kitchen weekly",
      "Vacuum every 3 days",
      "Clean refrigerator monthly",
    ],
  },
  {
    category: "Plants & Garden",
    color: "mint" as const,
    gradient: "from-[var(--pop-mint)] to-[var(--pop-cyan)]",
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 3v4M3 5h4M6 17v4m-2-2h4m5-16l2.286 6.857L21 12l-5.714 2.143L13 21l-2.286-6.857L5 12l5.714-2.143L13 3z" />
      </svg>
    ),
    tasks: [
      "Water indoor plants every 3 days",
      "Fertilize every 2 weeks",
      "Rotate plants monthly",
      "Check for pests weekly",
    ],
  },
  {
    category: "Pet Care",
    color: "orange" as const,
    gradient: "from-[var(--pop-orange)] to-[var(--pop-yellow)]",
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
      </svg>
    ),
    tasks: [
      "Groom dog every month",
      "Flea treatment every 30 days",
      "Vet checkup every 6 months",
      "Nail trim every 2 weeks",
    ],
  },
  {
    category: "Vehicle",
    color: "blue" as const,
    gradient: "from-[var(--pop-blue)] to-[var(--pop-purple)]",
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7h12m0 0l-4-4m4 4l-4 4m0 6H4m0 0l4 4m-4-4l4-4" />
      </svg>
    ),
    tasks: [
      "Oil change every 3 months",
      "Tire rotation every 6 months",
      "Car wash every 2 weeks",
      "Check tire pressure monthly",
    ],
  },
  {
    category: "Health & Wellness",
    color: "purple" as const,
    gradient: "from-[var(--pop-purple)] to-[var(--pop-pink)]",
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
      </svg>
    ),
    tasks: [
      "Take vitamins daily",
      "Dentist checkup every 6 months",
      "Replace toothbrush every 3 months",
      "Eye exam yearly",
    ],
  },
  {
    category: "Appliances",
    color: "cyan" as const,
    gradient: "from-[var(--pop-cyan)] to-[var(--pop-mint)]",
    icon: (
      <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
      </svg>
    ),
    tasks: [
      "Replace water filter every 6 months",
      "Descale coffee maker monthly",
      "Clean dishwasher monthly",
      "Check smoke detectors yearly",
    ],
  },
];

function UseCaseCard({ useCase, index }: { useCase: typeof useCases[0]; index: number }) {
  const ref = useRef(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });

  return (
    <motion.div
      ref={ref}
      initial={{ opacity: 0, y: 40, scale: 0.95 }}
      animate={isInView ? { opacity: 1, y: 0, scale: 1 } : { opacity: 0, y: 40, scale: 0.95 }}
      transition={{ duration: 0.5, delay: index * 0.1 }}
    >
      <Card className="h-full overflow-hidden group">
        {/* Header */}
        <div className={`bg-gradient-to-r ${useCase.gradient} -mx-6 -mt-6 px-6 py-4 mb-4`}>
          <div className="flex items-center gap-3 text-white">
            {useCase.icon}
            <h3 className="text-lg font-bold">{useCase.category}</h3>
          </div>
        </div>

        {/* Tasks */}
        <ul className="space-y-3">
          {useCase.tasks.map((task, i) => (
            <motion.li
              key={task}
              className="flex items-start gap-3"
              initial={{ opacity: 0, x: -20 }}
              animate={isInView ? { opacity: 1, x: 0 } : { opacity: 0, x: -20 }}
              transition={{ duration: 0.4, delay: index * 0.1 + i * 0.1 + 0.3 }}
            >
              <div className={`w-1.5 h-1.5 rounded-full mt-2 bg-gradient-to-r ${useCase.gradient}`} />
              <span className="text-[var(--muted)] text-sm">{task}</span>
            </motion.li>
          ))}
        </ul>
      </Card>
    </motion.div>
  );
}

export function UseCases() {
  const containerRef = useRef(null);
  const { scrollYProgress } = useScroll({
    target: containerRef,
    offset: ["start end", "end start"],
  });

  const y1 = useTransform(scrollYProgress, [0, 1], ["0%", "20%"]);
  const y2 = useTransform(scrollYProgress, [0, 1], ["0%", "-20%"]);

  return (
    <section
      id="use-cases"
      ref={containerRef}
      className="py-32 relative overflow-hidden"
    >
      {/* Background decorations */}
      <motion.div
        className="absolute top-0 left-0 w-[500px] h-[500px] bg-[var(--pop-coral)]/5 rounded-full blur-3xl"
        style={{ y: y1 }}
      />
      <motion.div
        className="absolute bottom-0 right-0 w-[600px] h-[600px] bg-[var(--pop-blue)]/5 rounded-full blur-3xl"
        style={{ y: y2 }}
      />

      <div className="max-w-7xl mx-auto px-6 relative z-10">
        {/* Section header */}
        <div className="text-center mb-16">
          <Badge variant="orange" className="mb-4">Use Cases</Badge>
          <h2 className="text-4xl md:text-5xl lg:text-6xl font-bold mb-6">
            <span>Track</span>{" "}
            <span className="gradient-text">everything</span>
            <br />
            <span>that matters.</span>
          </h2>
          <p className="text-xl text-[var(--muted)] max-w-2xl mx-auto">
            From household chores to personal health, DaysTracker keeps all your recurring tasks organized and visible.
          </p>
        </div>

        {/* Use cases grid */}
        <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
          {useCases.map((useCase, index) => (
            <UseCaseCard key={useCase.category} useCase={useCase} index={index} />
          ))}
        </div>

        {/* Bottom CTA */}
        <motion.div
          className="text-center mt-16"
          initial={{ opacity: 0, y: 20 }}
          whileInView={{ opacity: 1, y: 0 }}
          viewport={{ once: true }}
          transition={{ delay: 0.5 }}
        >
          <p className="text-xl text-[var(--muted)] mb-2">
            And so much more...
          </p>
          <p className="text-lg font-medium gradient-text">
            If it repeats, DaysTracker tracks it.
          </p>
        </motion.div>
      </div>
    </section>
  );
}
